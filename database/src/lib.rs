pub use crate::db::Database;
use crate::models::post::Post;
use crate::models::CURD;

mod db;
pub mod models;

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn db_test() {
    let db = Database::new().await.unwrap();
    let mut post = Post::new("Test post", "Hello world");
    post.insert(&db).await.unwrap();
    post.title = "wcnm".to_string();
    post.update(&db).await.unwrap();
    post.delete(&db).await.unwrap();
}
