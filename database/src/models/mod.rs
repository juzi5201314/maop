use crate::db::Database;
#[macro_export]
macro_rules! gen_model {
    ($name:ident, { $($(#[$attr:meta])* $field:ident: $r#type:ty),* }) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
        pub struct $name {
            $($(#[$attr])* pub $field: $r#type),*
        }
        impl $name {
            $(#[allow(dead_code)] pub fn $field(&self) -> &$r#type {
                &self.$field
            })*
        }
    };
}

utils::pub_mods!(post);

pub fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub trait Table {
    fn table_name() -> &'static str;
}

#[async_trait::async_trait]
pub trait CURD {
    async fn insert(&self, db: &Database) -> anyhow::Result<()>;
    async fn update(&self, db: &Database) -> anyhow::Result<()>;
    //async fn query<E>(execute: E, db: &AnyPool) -> anyhow::Result<Self> where E: Execute<_>;
    async fn delete(self, db: &Database) -> anyhow::Result<()>;
}
