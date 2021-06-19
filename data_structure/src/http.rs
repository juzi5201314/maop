use crate::wraps::Posts;
use database::Database;
use crate::website::WebsiteInfo;

#[derive(serde::Serialize)]
pub struct IndexData {
    website_info: WebsiteInfo,
    posts: Posts
}

impl IndexData {
    pub async fn new(db: &Database) -> anyhow::Result<Self> {
        Ok(IndexData {
            website_info: WebsiteInfo::new()?,
            posts: Posts::get(db).await?
        })
    }
}
