use anyhow::Context;
use chrono::NaiveDateTime;
use sea_orm::prelude::DbConn;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait,
    DeriveEntityModel, DeriveIntoActiveModel, DerivePrimaryKey,
    DeriveRelation, EntityTrait, EnumIter, IdenStatic,
    IntoActiveModel, PrimaryKeyTrait, QueryFilter, Related,
    RelationDef, RelationTrait,
};

use crate::models::comment::{
    AsCommentId, Comment, CommentModel, NewComment,
};

use super::def_fn;

pub type Post = Entity;
pub type PostModel = Model;

#[derive(
    Clone,
    Debug,
    PartialEq,
    DeriveEntityModel,
    serde::Serialize,
    serde::Deserialize,
)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub title: String,
    pub content: String,

    pub create_time: NaiveDateTime,
    pub last_modified_time: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

#[derive(DeriveIntoActiveModel)]
struct DeletePost {
    id: u32,
}

#[derive(DeriveIntoActiveModel)]
pub struct NewPost {
    pub title: String,
    pub content: String,
}

impl Post {
    def_fn!(
        find_all(db) -> Vec<PostModel> {
            Post::find().all(db).await.context("Post::find_all")
        }
    );

    def_fn!(
        recover(db, posts: Vec<PostModel>) -> () {
            Post::delete_many()
                .exec(db)
                .await
                .context("Post::recover::delete_all")?;
            for post in posts {
                let active_model = Into::<ActiveModel>::into(post);
                active_model.insert(db).await.context("Post::recover::insert")?;
            }
            Ok(())
        }
    );

    def_fn!(
        delete(db, id: u32) -> () {
            Comment::delete_many()
                .filter(super::comment::Column::PostId.eq(id))
                .exec(db)
                .await
                .context("Post::delete::Comment::delete_many")?;

            (DeletePost {
                id
            }).into_active_model()
                .delete(db)
                .await
                .map(|_| {})
                .context("Post::delete")
        }
    );

    def_fn!(
        find_one(db, id: u32) -> Option<PostModel> {
            Post::find_by_id(id).one(db).await.context("Post::find_one")
        }
    );

    def_fn!(
        find_and_commit(db, id: u32) -> Option<(PostModel, Vec<CommentModel>)> {
            Post::find_by_id(id)
            .find_with_related(super::comment::Entity)
            .all(db)
            .await
            .map(|vec| vec.into_iter().next())
            .context("Post::find_and_commit")
        }
    );

    def_fn!(
        insert(db, new_post: NewPost) -> u32 {
            let now = chrono::Local::now().naive_local();
            let mut active_model = new_post.into_active_model();
            active_model.create_time = ActiveValue::set(now);
            active_model.last_modified_time = ActiveValue::set(now);
            active_model
                .insert(db)
                .await
                .map(|am: ActiveModel| am.id.unwrap())
                .context("Post::insert")
        }
    );

    def_fn!(
        update(db, id: u32, title: Option<String>, content: Option<String>) -> () {
            let now = chrono::Local::now().naive_local();
            (ActiveModel {
                id: ActiveValue::set(id),
                title: title.map(ActiveValue::set).unwrap_or_else(ActiveValue::unset),
                content: content.map(ActiveValue::set).unwrap_or_else(ActiveValue::unset),
                last_modified_time: ActiveValue::set(now),
                ..Default::default()
            }).into_active_model()
                .update(db)
                .await
                .map(|_| ())
                .context("Post::update")
        }
    );

    def_fn!(
        reply(db, id: u32, new_comment: NewComment, reply_to: Option<u32>) -> u32 {
            Comment::insert(db, id, new_comment, reply_to).await.context("Post::reply")
        }
    );
}

impl PostModel {
    #[inline]
    pub async fn refresh(
        &mut self,
        db: &DbConn,
    ) -> anyhow::Result<()> {
        *self = Post::find_one(db, self.id).await?.unwrap();
        Ok(())
    }

    #[inline]
    pub async fn delete(self, db: &DbConn) -> anyhow::Result<()> {
        Post::delete(db, self.id).await
    }

    #[inline]
    pub async fn update(
        &self,
        db: &DbConn,
        title: Option<String>,
        content: Option<String>,
    ) -> anyhow::Result<()> {
        Post::update(db, self.id, title, content).await
    }

    #[inline]
    pub async fn reply<C>(
        &self,
        db: &DbConn,
        new_comment: NewComment,
        reply_to: Option<C>,
    ) -> anyhow::Result<u32>
    where
        C: AsCommentId,
    {
        Post::reply(
            db,
            self.id,
            new_comment,
            reply_to.map(|c| c.as_comment_id()),
        )
        .await
    }
}
