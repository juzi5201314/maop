

use crate::models::{CURD, Table, unique_id};
use crate::db::Database;
use std::ops::Deref;

gen_model!(Post, {
    id: String,

    title: String,
    content: String,

    create_time: chrono::DateTime<chrono::Local>,
    last_modified_time: chrono::DateTime<chrono::Local>
});

impl Post {
    pub fn new<S>(title: S, content: S) -> Self where S: Into<String> {
        Post {
            id: unique_id(),
            title: title.into(),
            content: content.into(),
            create_time: chrono::Local::now(),
            last_modified_time: chrono::Local::now()
        }
    }

    pub async fn query_all(db: &Database) -> anyhow::Result<Vec<Self>> {
        Ok(
            sqlx::query_as("SELECT * FROM post")
                .fetch_all(db.deref())
                .await?
        )
    }

    pub async fn query_from_id<S>(id: S, db: &Database) -> anyhow::Result<Self> where S: AsRef<str> {
        Ok(sqlx::query_as::<_, Self>("SELECT * FROM post WHERE id = ?")
            .bind(id.as_ref())
            .fetch_one(db.deref())
            .await?)
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.last_modified_time = chrono::Local::now();
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.last_modified_time = chrono::Local::now();
    }
}

impl Table for Post {
    fn table_name() -> &'static str {
        "post"
    }
}

#[async_trait::async_trait]
impl CURD for Post {
    async fn insert(&self, db: &Database) -> anyhow::Result<()> {
        sqlx::query("\
        INSERT INTO post (id, title, content, create_time, last_modified_time) \
        VALUES (?, ?, ?, ?, ?)\
        ")
            .bind(&self.id)
            .bind(&self.title)
            .bind(&self.content)
            .bind(&self.create_time)
            .bind(&self.last_modified_time)
            .execute(db.deref())
            .await?;
        Ok(())
    }

    async fn update(&self, db: &Database) -> anyhow::Result<()>  {
        sqlx::query("\
        UPDATE post \
        SET title = ?, content = ?, create_time = ?, last_modified_time = ? \
        WHERE id = ? \
        ")
            .bind(&self.title)
            .bind(&self.content)
            .bind(&self.create_time)
            .bind(&self.last_modified_time)
            .bind(&self.id)
            .execute(db.deref())
            .await?;
        Ok(())
    }

   /* async fn query(, db: &AnyPool) -> anyhow::Result<Self> {

    }*/

    async fn delete(self, db: &Database) -> anyhow::Result<()>  {
        sqlx::query("DELETE FROM post WHERE id = ?")
            .bind(self.id)
            .execute(db.deref())
            .await?;
        Ok(())
    }
}
