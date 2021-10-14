pub use crate::db::new;

mod db;
pub mod models;

#[cfg(test)]
mod test {
    use crate::db;
    use crate::models::comment::{Comments, NewComment};
    use crate::models::post::{NewPost, Posts};
    use rbatis::rbatis::Rbatis;

    #[tokio::test]
    async fn comment_curd_test() {
        let conf = config::get_config();
        let rb: Rbatis = db::new(&conf).await.unwrap();

        let post: Posts = Posts::insert(
            &rb,
            NewPost {
                title: "title",
                content: "content",
            },
        )
        .await
        .unwrap();

        let hello_comment: Comments = post
            .reply(
                &rb,
                NewComment {
                    content: "hello",
                    nickname: "God",
                    email: "god@exmaple.com",
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(hello_comment.content, "hello");
        assert_eq!(hello_comment.post_id, post.id);

        let world_comment: Comments = hello_comment
            .reply_to(
                &rb,
                NewComment {
                    content: "world",
                    nickname: "Adam",
                    email: "adam@exmaple.com",
                },
            )
            .await
            .unwrap();
        assert_eq!(world_comment.content, "world");

        assert_eq!(world_comment.parent_id, Some(hello_comment.id));

        assert_eq!(
            hello_comment
                .query_replies(&rb)
                .await
                .unwrap()
                .first()
                .map(|reply| reply.id),
            Some(world_comment.id)
        );
    }

    #[tokio::test]
    async fn post_curd_test() {
        let conf = config::get_config();
        let rb: Rbatis = db::new(&conf).await.unwrap();

        // insert
        let mut post: Posts = Posts::insert(
            &rb,
            NewPost {
                title: "title",
                content: "content",
            },
        )
        .await
        .unwrap();
        assert_eq!(post.title, "title");
        assert_eq!(post.content, "content");

        // update
        post.update(
            &rb,
            post.id,
            NewPost {
                title: "new title",
                content: "new content",
            },
        )
        .await
        .unwrap();
        assert_eq!(post.title, "new title");
        assert_eq!(post.content, "new content");

        // query
        let new_post: Posts =
            Posts::select(&rb, post.id).await.unwrap();
        assert_eq!(new_post.title, post.title);
        assert_eq!(new_post.content, post.content);
        assert_eq!(new_post.create_time, post.create_time);

        // remove
        new_post.remove(&rb).await.unwrap();
        assert!(Posts::select(&rb, post.id).await.is_err());
    }
}
