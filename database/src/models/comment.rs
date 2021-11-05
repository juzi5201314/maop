use anyhow::Context;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::DbConn;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait,
    DeriveEntityModel, DeriveIntoActiveModel, DerivePrimaryKey,
    EntityTrait, EnumIter, IdenStatic, IntoActiveModel,
    PrimaryKeyTrait, QueryFilter, QueryOrder, Related, RelationDef,
    RelationTrait,
};

use super::def_fn;

pub type Comment = Entity;
pub type CommentModel = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,

    /// 对应的文章
    pub post_id: u32,
    pub content: String,
    pub create_time: NaiveDateTime,
    /// 发布者邮箱
    pub email: String,
    /// 发布者昵称
    pub nickname: String,

    /// 回复的评论id
    #[sea_orm(nullable)]
    pub parent_id: Option<u32>,

    /// 已经删除(对用户而言)
    pub deleted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Post,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Post => Entity::belongs_to(super::post::Entity)
                .from(Column::PostId)
                .to(super::post::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

#[derive(DeriveIntoActiveModel)]
pub struct NewComment {
    pub nickname: String,
    pub email: String,
    pub content: String,
}

#[derive(DeriveIntoActiveModel)]
struct DeleteComment {
    id: u32,
}

pub trait AsCommentId {
    fn as_comment_id(&self) -> u32;
}

impl AsCommentId for &CommentModel {
    fn as_comment_id(&self) -> u32 {
        self.post_id
    }
}

impl AsCommentId for u32 {
    fn as_comment_id(&self) -> u32 {
        *self
    }
}

impl Comment {
    def_fn!(
        find_all(db) -> Vec<CommentModel> {
            Comment::find().all(db).await.context("Comment::find_all")
        }
    );

    def_fn!(
        recover(db, comments: Vec<CommentModel>) -> () {
            Comment::delete_many()
                .exec(db)
                .await
                .context("Comment::recover::delete_all")?;
            for comment in comments {
                let active_model = Into::<ActiveModel>::into(comment);
                active_model.insert(db).await.context("Comment::recover::insert")?;
            }
            Ok(())
        }
    );

    def_fn!(
        hard_delete(db, id: u32) -> () {
            (DeleteComment {
                id
            }).into_active_model()
                .delete(db)
                .await
                .map(|_| {})
                .context("Comment::hard_delete")
        }
    );

    def_fn!(
        soft_delete(db, id: u32) -> () {
            (ActiveModel {
                id: ActiveValue::set(id),
                deleted: ActiveValue::set(true),
                ..Default::default()
            }).into_active_model()
                .delete(db)
                .await
                .map(|_| {})
                .context("Comment::soft_delete")
        }
    );

    def_fn!(
        find_one(db, id: u32) -> Option<CommentModel> {
            Comment::find_by_id(id).one(db).await.context("Comment::find_one")
        }
    );

    def_fn!(
        find_replies(db, id: u32) -> Vec<CommentModel> {
            Comment::find()
            .filter(Column::ParentId.contains(&id.to_string()))
            .order_by_desc(Column::CreateTime)
            .all(db)
            .await
            .context("Comment::find_replies")
        }
    );

    def_fn!(
        insert(db, post_id: u32, new_comment: NewComment, reply_to: Option<u32>) -> u32 {
            let now = chrono::Local::now().naive_local();
            let mut active_model = new_comment.into_active_model();

            active_model.post_id = ActiveValue::set(post_id);
            active_model.create_time = ActiveValue::set(now);
            active_model.deleted = ActiveValue::set(false);
            active_model.parent_id = ActiveValue::set(reply_to);
            active_model.into_active_model()
                .insert(db)
                .await
                .map(|am: ActiveModel| am.id.unwrap())
                .context("Comment::insert")
        }
    );
}

impl CommentModel {
    #[inline]
    pub async fn reply(
        &self,
        db: &DbConn,
        new_comment: NewComment,
    ) -> anyhow::Result<u32> {
        Comment::insert(
            db,
            self.post_id,
            new_comment,
            Some(self.id),
        )
        .await
    }
}
