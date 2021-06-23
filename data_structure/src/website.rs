#[derive(serde::Serialize)]
pub struct WebsiteInfo {
    name: String,
    author: String,
}

impl WebsiteInfo {
    pub fn new() -> anyhow::Result<Self> {
        Ok(WebsiteInfo {
            name: settings::website_name()?,
            author: settings::website_author_name()?,
        })
    }
}
