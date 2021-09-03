pub use crate::db::new;

mod db;
pub mod models;

#[cfg(test)]
mod test {
    use std::env::set_var;
    use rbatis::rbatis::Rbatis;
    use crate::db;
    use crate::models::post::{Posts, NewPost};
    use crate::models::commit::{Commits, NewCommit};

    #[tokio::test]
    async fn commit_curd_test() {
        set_var("maop_database_uri", "sqlite::memory:");
        let conf = config::get_config();
        let rb: Rbatis = db::new(conf.database()).await.unwrap();

        let mut post: Posts = Posts::insert(&rb, NewPost {
            title: "title",
            content: "content"
        }).await.unwrap();

        let hello_commit: Commits = post.reply(&rb, NewCommit {
            content: "hello",
            nickname: "God",
            email: "god@exmaple.com"
        }, None).await.unwrap();
        assert_eq!(hello_commit.content, "hello");
        assert_eq!(hello_commit.post_id, post.id);

        let world_commit: Commits = hello_commit.reply_to(&rb, NewCommit {
            content: "world",
            nickname: "Adam",
            email: "adam@exmaple.com"
        }).await.unwrap();
        assert_eq!(world_commit.content, "world");

        assert_eq!(world_commit.parent_id, Some(hello_commit.id));

        assert_eq!(hello_commit.query_replies(&rb).await.unwrap().first().map(|reply| reply.id), Some(world_commit.id));
    }

    #[tokio::test]
    async fn post_curd_test() {
        set_var("maop_database_uri", "sqlite::memory:");
        let conf = config::get_config();
        let rb: Rbatis = db::new(conf.database()).await.unwrap();

        // insert
        let mut post: Posts = Posts::insert(&rb, NewPost {
            title: "title",
            content: "content"
        }).await.unwrap();
        assert_eq!(post.title, "title");
        assert_eq!(post.content, "content");

        // update
        post.update(&rb, post.id, NewPost {
            title: "new title",
            content: "new content"
        }).await.unwrap();
        assert_eq!(post.title, "new title");
        assert_eq!(post.content, "new content");

        // query
        let new_post: Posts = Posts::select(&rb, post.id).await.unwrap();
        assert_eq!(new_post.title, post.title);
        assert_eq!(new_post.content, post.content);
        assert_eq!(new_post.create_time, post.create_time);

        // remove
        new_post.remove(&rb).await.unwrap();
        assert!(Posts::select(&rb, post.id).await.is_err());
    }

}

