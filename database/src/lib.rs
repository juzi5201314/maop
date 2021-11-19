#![feature(try_blocks)]
#![feature(type_ascription)]
#![feature(decl_macro)]

pub use crate::db::new;

mod db;
pub mod models;

#[cfg(test)]
mod test {
    use crate::db;
    use crate::models::comment::Comment;
    use crate::models::comment::NewComment;
    use crate::models::post::{NewPost, Post};

    #[tokio::test]
    async fn comment_curd_test() {
        config::init(vec![]).unwrap();
        let db = db::new().await.unwrap();

        let post_id = Post::insert(
            &db,
            NewPost {
                title: "title".to_owned(),
                content: "content".to_owned(),
            },
        )
        .await
        .unwrap();

        let post =
            Post::find_one(&db, post_id).await.unwrap().unwrap();
        assert_eq!(post_id, post.id);

        let hello_comment_id = post
            .reply::<u32>(
                &db,
                NewComment {
                    content: "hello".to_owned(),
                    nickname: "God".to_owned(),
                    email: "god@exmaple.com".to_owned(),
                },
                None,
            )
            .await
            .unwrap();

        let hello_comment = Comment::find_one(&db, hello_comment_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(hello_comment_id, hello_comment.id);
        assert_eq!(hello_comment.content, "hello");
        assert_eq!(hello_comment.post_id, post.id);

        let world_comment_id = hello_comment
            .reply(
                &db,
                NewComment {
                    content: "world".to_owned(),
                    nickname: "Adam".to_owned(),
                    email: "adam@exmaple.com".to_owned(),
                },
            )
            .await
            .unwrap();

        let world_comment = Comment::find_one(&db, world_comment_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(world_comment_id, world_comment.id);
        assert_eq!(world_comment.content, "world");
        assert_eq!(world_comment.post_id, post.id);
        assert_eq!(world_comment.parent_id, Some(hello_comment.id));

        assert_eq!(
            Comment::find_replies(&db, hello_comment.id)
                .await
                .unwrap(),
            vec![world_comment]
        );
    }

    #[tokio::test]
    async fn post_curd_test() {
        config::init(vec![]).unwrap();
        let db = db::new().await.unwrap();

        let post_id = Post::insert(
            &db,
            NewPost {
                title: "title".to_owned(),
                content: "content".to_owned(),
            },
        )
        .await
        .unwrap();
        let mut post =
            Post::find_one(&db, post_id).await.unwrap().unwrap();

        assert_eq!(post.title, "title");
        assert_eq!(post.content, "content");

        post.update(
            &db,
            Some("new title".to_owned()),
            Some("new content".to_owned()),
        )
        .await
        .unwrap();
        post.refresh(&db).await.unwrap();

        assert_eq!(post.title, "new title");
        assert_eq!(post.content, "new content");

        let id = post.id;
        post.delete(&db).await.unwrap();
        assert!(Post::find_one(&db, id).await.unwrap().is_none());
    }
}
