use chrono::NaiveDateTime;
use compact_str::CompactStr;
use rbatis::core::value::DateTimeNow;
use rbatis::crud::CRUDMut;
use rbatis::crud::{Skip, CRUD};
use rbatis::crud_table;
use rbatis::rbatis::Rbatis;

use error::Error;

#[derive(Default, Clone)]
pub struct NewComment<S1, S2, S3>
where
    S1: Into<CompactStr> + Clone,
    S2: Into<CompactStr> + Clone,
    S3: Into<CompactStr> + Clone,
{
    pub content: S1,
    pub nickname: S2,
    pub email: S3,
}

#[crud_table]
pub struct Comments {
    pub id: u64,

    /// 对应的文章
    pub post_id: u64,
    pub content: CompactStr,
    pub create_time: NaiveDateTime,
    /// 发布者邮箱
    pub email: CompactStr,
    /// 发布者昵称
    pub nickname: CompactStr,

    /// 回复的评论id
    pub parent_id: Option<u64>,

    /// 已经删除(对用户而言)
    #[serde(serialize_with = "serialize")]
    #[serde(deserialize_with = "deserialize")]
    pub deleted: bool,
}

use serde::{Deserialize, Deserializer, Serializer};

fn serialize<S>(val: &bool, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    ser.serialize_u8(*val as u8)
}

fn deserialize<'de, D>(der: D) -> Result<bool, D::Error> where D: Deserializer<'de> {
    Ok(u8::deserialize(der)? != 0)
}

impl Comments {
    #[inline]
    pub async fn query_all(rb: &Rbatis) -> Result<Vec<Self>, Error> {
        rb.fetch_list().await.map_err(Into::into)
    }

    /// 只是把评论替换为`该评论已删除`.
    /// 并没有删除该评论
    pub async fn soft_delete(
        rb: &Rbatis,
        id: u64,
    ) -> Result<(), Error> {
        rb.update_by_wrapper::<Self>(
            &Comments {
                id,
                post_id: 0,
                parent_id: Option::None,
                create_time: NaiveDateTime::now(),
                content: CompactStr::default(),
                email: CompactStr::default(),
                nickname: CompactStr::default(),
                deleted: true,
            },
            rb.new_wrapper().eq("id", id),
            &[
                Skip::Column("id"),
                Skip::Column("post_id"),
                Skip::Column("create_time"),
                Skip::Column("parent_id"),
                Skip::Column("content"),
                Skip::Column("email"),
                Skip::Column("nickname"),
            ],
        )
        .await?;
        Ok(())
    }

    /// 删除评论并且删除全部回复该评论的评论
    pub async fn hard_delete(
        rb: &Rbatis,
        id: u64,
    ) -> Result<(), Error> {
        rb.remove_by_wrapper::<Self>(
            rb.new_wrapper().eq("id", id).or().eq("parent_id", id),
        )
        .await?;
        Ok(())
    }

    #[inline]
    pub async fn select(rb: &Rbatis, id: u64) -> Result<Self, Error> {
        rb.fetch_by_column("id", &id).await.map_err(Into::into)
    }

    pub async fn insert<S1, S2, S3>(
        rb: &Rbatis,
        post_id: u64,
        new_comment: NewComment<S1, S2, S3>,
        reply_to: Option<u64>,
    ) -> Result<Self, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
        S3: Into<CompactStr> + Clone,
    {
        let mut tx = rb.acquire_begin().await?;

        let last_id = tx
            .save(
                &Comments {
                    id: 0,
                    post_id,
                    content: new_comment.content.into(),
                    create_time: NaiveDateTime::now(),
                    email: new_comment.email.into(),
                    nickname: new_comment.nickname.into(),
                    parent_id: reply_to,
                    deleted: false,
                },
                &[Skip::Column("id")],
            )
            .await?
            .last_insert_id
            .unwrap();
        let comment = tx.fetch_by_column("id", &last_id).await?;
        tx.commit().await?;
        Ok(comment)
    }

    #[inline]
    pub async fn reply_to<S1, S2, S3>(
        &self,
        rb: &Rbatis,
        new_comment: NewComment<S1, S2, S3>,
    ) -> Result<Self, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
        S3: Into<CompactStr> + Clone,
    {
        Self::insert(rb, self.post_id, new_comment, Some(self.id))
            .await
    }

    #[inline]
    pub async fn query_replies(
        &self,
        rb: &Rbatis,
    ) -> Result<Vec<Self>, Error> {
        rb.fetch_list_by_wrapper(
            rb.new_wrapper().eq("parent_id", self.id),
        )
        .await
        .map_err(Into::into)
    }
}
