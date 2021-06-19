use crate::wraps::Posts;
use database::Database;

#[derive(serde::Serialize)]
pub struct IndexData {
    posts: Posts
}

impl IndexData {
    pub async fn new(db: &Database) -> anyhow::Result<Self> {
        Ok(IndexData {
            posts: Posts::get(db).await?
        })
    }
}
