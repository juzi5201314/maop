use std::collections::HashMap;
use std::fs::{create_dir, create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_session::{Session, SessionStore as AsyncSessionStore};
use compact_str::CompactStr;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

#[cfg(not(feature = "session_store_rocksdb"))]
pub type SessionStore = FileStore;
#[cfg(feature = "session_store_rocksdb")]
pub type SessionStore = rocksdb::RocksdbStore;

#[derive(Debug, Clone)]
pub struct FileStore {
    cache: Arc<RwLock<HashMap<CompactStr, Session>>>,
    path: PathBuf,
}

impl FileStore {
    #[allow(unused)]
    pub async fn new<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        create_dir_all(path.as_ref())?;
        Ok(FileStore {
            cache: Arc::new(RwLock::new(HashMap::new())),
            path: path.as_ref().to_path_buf(),
        })
    }

    async fn store(&self, session: &Session) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.path.join(session.id()))
            .await?;
        file.write_all(&bincode::serialize(session)?).await?;
        file.sync_data().await?;
        Ok(())
    }

    async fn load(
        &self,
        session_id: &str,
    ) -> anyhow::Result<Option<Session>> {
        let path = self.path.join(session_id);
        Ok(if path.exists() {
            let mut file: File =
                OpenOptions::new().read(true).open(path).await?;
            let mut data = Vec::with_capacity(200);
            file.read_to_end(&mut data).await?;
            bincode::deserialize(&*data)?
        } else {
            None
        })
    }
}

#[async_trait::async_trait]
impl AsyncSessionStore for FileStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(if let Some(session) = self.cache.read().await.get(&*id) {
            Some(session.clone())
        } else {
            self.load(&id).await?.and_then(Session::validate)
        })
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> async_session::Result<Option<String>> {
        self.store(&session).await?;
        self.cache
            .write()
            .await
            .insert(session.id().into(), session.clone());
        // session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(
        &self,
        session: Session,
    ) -> async_session::Result {
        self.cache.write().await.remove(session.id());
        remove_file(self.path.join(session.id()))?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        self.cache.write().await.clear();
        remove_dir_all(&self.path)
            .and_then(|_| create_dir(&self.path))?;
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
