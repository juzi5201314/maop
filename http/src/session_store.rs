use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_session::{Session, SessionStore as AsyncSessionStore};
use compact_str::CompactStr;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};

#[cfg(not(feature = "session_store_rocksdb"))]
pub type SessionStore = FileStore;
#[cfg(feature = "session_store_rocksdb")]
pub type SessionStore = rocksdb::RocksdbStore;

#[derive(Debug, Clone)]
pub struct FileStore {
    inner: Arc<RwLock<HashMap<CompactStr, Session>>>,
    file: Arc<Mutex<File>>,
    loaded: Arc<AtomicBool>,
}

impl FileStore {
    #[allow(unused)]
    pub async fn new<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(FileStore {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file: Arc::new(Mutex::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .read(true)
                    .truncate(true)
                    .open(path)
                    .await?,
            )),
            loaded: Arc::new(AtomicBool::new(false)),
        })
    }

    async fn store(
        &self,
        data: &HashMap<CompactStr, Session>,
    ) -> anyhow::Result<()> {
        let mut file = self.file.lock().await;
        file.set_len(0).await?;
        file.write_all(&bincode::serialize(data)?).await?;
        file.sync_data().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncSessionStore for FileStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<Session>> {
        if !self.loaded.swap(true, Ordering::SeqCst) {
            let mut data = Vec::with_capacity(128);
            self.file.lock().await.read_to_end(&mut data).await?;
            *self.inner.write().await = bincode::deserialize(&data)?;
        }
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(self
            .inner
            .read()
            .await
            .get(&*id)
            .cloned()
            .and_then(Session::validate))
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> async_session::Result<Option<String>> {
        let id: CompactStr = session.id().into();
        let mut inner = self.inner.write().await;

        inner.insert(id, session.clone());

        self.store(&*inner).await?;

        // session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(
        &self,
        session: Session,
    ) -> async_session::Result {
        let mut inner = self.inner.write().await;
        inner.remove(session.id());
        self.store(&*inner).await?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        let mut inner = self.inner.write().await;
        inner.clear();
        self.store(&*inner).await?;
        Ok(())
    }
}

#[cfg(feature = "session_store_rocksdb")]
mod rocksdb {
    use std::path::Path;
    use std::sync::Arc;

    use async_session::{Session, SessionStore};
    use rocksdb::{IteratorMode, Options, DB};

    #[derive(Debug, Clone)]
    pub struct RocksdbStore {
        inner: Arc<DB>,
    }

    impl RocksdbStore {
        pub async fn new<P>(path: P) -> Result<Self, rocksdb::Error>
        where
            P: AsRef<Path>,
        {
            Ok(RocksdbStore {
                inner: Arc::new(DB::open(
                    &{
                        let mut opt = Options::default();
                        opt.create_if_missing(true);
                        opt.set_keep_log_file_num(100);
                        opt.set_max_log_file_size(1024 ^ 2);
                        opt.set_recycle_log_file_num(100);
                        opt.create_missing_column_families(true);
                        opt
                    },
                    path,
                )?),
            })
        }
    }

    #[async_trait::async_trait]
    impl SessionStore for RocksdbStore {
        async fn load_session(
            &self,
            cookie_value: String,
        ) -> async_session::Result<Option<Session>> {
            let id = Session::id_from_cookie_value(&cookie_value)?;
            Ok(match self.inner.get(id)? {
                None => None,
                Some(data) => Some(bincode::deserialize(&data)?),
            })
        }

        async fn store_session(
            &self,
            session: Session,
        ) -> async_session::Result<Option<String>> {
            self.inner
                .put(session.id(), bincode::serialize(&session)?)?;
            Ok(session.into_cookie_value())
        }

        async fn destroy_session(
            &self,
            session: Session,
        ) -> async_session::Result {
            self.inner.delete(session.id()).map_err(Into::into)
        }

        async fn clear_store(&self) -> async_session::Result {
            for (key, _) in
                self.inner.full_iterator(IteratorMode::Start)
            {
                self.inner.delete(key)?
            }

            Ok(())
        }
    }
}
