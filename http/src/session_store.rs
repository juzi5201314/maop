use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{
    create_dir, create_dir_all, read_dir, remove_dir_all, remove_file,
};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_session::{Session, SessionStore as AsyncSessionStore};
use compact_str::CompactStr;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use timer::{Follow, Task};

#[cfg(not(feature = "session_store_rocksdb"))]
pub type SessionStore = FileStore;
#[cfg(feature = "session_store_rocksdb")]
pub type SessionStore = rocksdb::RocksdbStore;

#[derive(Debug, Clone)]
pub struct FileStore {
    cache: Arc<Mutex<HashMap<CompactStr, Session>>>,
    path: PathBuf,
}

impl FileStore {
    #[allow(unused)]
    pub async fn new<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        create_dir_all(path.as_ref())?;
        FileStore::regularly_check_expired(path.as_ref());
        Ok(FileStore {
            cache: Arc::new(Mutex::new(HashMap::new())),
            path: path.as_ref().to_path_buf(),
        })
    }

    async fn store(&self, session: &Session) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.path.join(FileStore::id_hash(session.id())))
            .await?;
        file.write_all(&bincode::serialize(session)?).await?;
        file.sync_data().await?;
        Ok(())
    }

    async fn load(
        &self,
        session_id: &str,
    ) -> anyhow::Result<Option<Session>> {
        let path = self.path.join(FileStore::id_hash(session_id));
        Ok(if path.exists() {
            let mut file: File =
                OpenOptions::new().read(true).open(path).await?;
            let mut data = Vec::with_capacity(200);
            file.read_to_end(&mut data).await?;
            Some(bincode::deserialize(&*data)?)
        } else {
            None
        })
    }

    fn regularly_check_expired(path: &Path) {
        let path = path.to_path_buf();
        global_resource::TIME_WHEEL.add_task(Task::interval(
            move || {
                log::debug!("checking for expired session");

                let path = path.clone();
                Box::pin(async {
                    if let Result::<(), anyhow::Error>::Err(err) = try {
                        for dir_entry in read_dir(path)? {
                            let dir = dir_entry?.path();

                            let mut file: File =
                                OpenOptions::new().read(true).open(&dir).await?;
                            let mut data = Vec::with_capacity(200);
                            file.read_to_end(&mut data).await?;

                            let session = bincode::deserialize::<Session>(&*data)?;
                            if session.is_expired() {
                                remove_file(
                                    &dir,
                                )?;
                                log::info!("Deleted the expired session: {}", dir.display());
                            }
                        }
                    } {
                        log::error!("regularly_check_expired: {:?}", err);
                    }

                    Follow::Done
                })
            },
            *config::get_config_temp().http().overdue_check_interval().duration(),
        ));
    }

    fn id_hash(id: &str) -> String {
        let mut hasher = DefaultHasher::default();
        id.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

#[async_trait::async_trait]
impl AsyncSessionStore for FileStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let mut cache = self.cache.lock().await;
        Ok(if let Some(session) = cache.get(&*id) {
            Some(session.clone())
        } else {
            let session =
                self.load(&id).await?.and_then(Session::validate);
            if let Some(session) = &session {
                cache.insert(id.into(), session.clone());
            }
            session
        })
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> async_session::Result<Option<String>> {
        self.store(&session).await?;
        self.cache
            .lock()
            .await
            .insert(session.id().into(), session.clone());
        // session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(
        &self,
        session: Session,
    ) -> async_session::Result {
        self.cache.lock().await.remove(session.id());
        remove_file(
            self.path.join(FileStore::id_hash(session.id())),
        )?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        self.cache.lock().await.clear();
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

    use timer::{Follow, Task};

    #[derive(Debug, Clone)]
    pub struct RocksdbStore {
        inner: Arc<DB>,
    }

    impl RocksdbStore {
        pub async fn new<P>(path: P) -> Result<Self, rocksdb::Error>
        where
            P: AsRef<Path>,
        {
            let inner = Arc::new(DB::open(
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
            )?);
            RocksdbStore::regularly_check_expired(
                path.as_ref(),
                &inner,
            );
            Ok(RocksdbStore { inner })
        }

        fn regularly_check_expired(path: &Path, db: &Arc<DB>) {
            let path = path.to_path_buf();
            global_resource::TIME_WHEEL.add_task(Task::interval(
                move || {
                    log::debug!("checking for expired session");

                    let path = path.clone();
                    let db = Arc::clone(&db);
                    Box::pin(async {
                        if let Result::<(), anyhow::Error>::Err(err) = try {
                            for (key, val) in db.full_iterator(IteratorMode::Start)
                            {
                                let session = bincode::deserialize::<Session>(&val)?;
                                if session.is_expired() {
                                    db.delete(key)?;
                                    log::info!("Deleted the expired session: {}", session.id());
                                }
                            }
                        } {
                            log::error!("regularly_check_expired: {:?}", err);
                        }
                        Follow::Done
                    })
                },
                *config::get_config_temp().http().overdue_check_interval().duration(),
            ));
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
