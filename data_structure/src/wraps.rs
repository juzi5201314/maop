use serde::{Deserialize, Serialize};

use config::{get_config, RocketConfig};
use database::models::post::Post;
use database::Database;

macro_rules! wrap {
    ($from:ty, $to:ident) => {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(transparent)]
        pub struct $to {
            inner: $from,
        }

        impl From<$from> for $to {
            fn from(inner: $from) -> Self {
                $to { inner }
            }
        }
    };
}
wrap!(Vec<Post>, Posts);
impl Posts {
    pub async fn get(db: &Database) -> anyhow::Result<Self> {
        Post::query_all(db).await.map(Posts::from)
    }
}

wrap!(RocketConfig, HttpServerConfig);
impl HttpServerConfig {
    pub fn get() -> Self {
        get_config().rocket().clone().into()
    }
}
