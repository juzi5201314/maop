utils::pub_mods!(post, comment);

macro def_fn {
    ($name:ident ($db:tt $(,)? $($param:ident: $ty:ty),*) -> $r:ty $body:block) => {
        #[inline]
        pub async fn $name($db: &sea_orm::DatabaseConnection, $($param: $ty),*) -> anyhow::Result<$r> {
            $body
        }
    },
    ($name:ident ($db:tt $(,)? $(mut $param:ident: $ty:ty),*) -> $r:ty $body:block) => {
        #[inline]
        pub async fn $name($db: &sea_orm::DatabaseConnection, $(mut $param: $ty),*) -> anyhow::Result<$r> {
            $body
        }
    },
}
