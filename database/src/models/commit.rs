use chrono::NaiveDateTime;
use compact_str::CompactStr;
use rbatis::core::value::DateTimeNow;
use rbatis::crud::CRUDMut;
use rbatis::crud::{Skip, CRUD};
use rbatis::crud_table;
use rbatis::rbatis::Rbatis;

use error::Error;

#[derive(Default, Clone)]
pub struct NewCommit<S1, S2, S3>
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
pub struct Commits {
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
}

impl Commits {
    #[inline]
    pub async fn query_all(rb: &Rbatis) -> Result<Vec<Self>, Error> {
        rb.fetch_list().await.map_err(Into::into)
    }

    #[inline]
    pub async fn remove_by_id(
        rb: &Rbatis,
        id: u64,
    ) -> Result<(), Error> {
        rb.remove_by_column::<Self, _>("id", &id).await?;
        Ok(())
    }

    #[inline]
    pub async fn remove(self, rb: &Rbatis) -> Result<(), Error> {
        Self::remove_by_id(rb, self.id).await?;
        Ok(())
    }

    #[inline]
    pub async fn select(rb: &Rbatis, id: u64) -> Result<Self, Error> {
        rb.fetch_by_column("id", &id).await.map_err(Into::into)
    }

    pub async fn insert<S1, S2, S3>(
        rb: &Rbatis,
        post_id: u64,
        new_commit: NewCommit<S1, S2, S3>,
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
                &Commits {
                    id: 0,
                    post_id,
                    content: new_commit.content.into(),
                    create_time: NaiveDateTime::now(),
                    email: new_commit.email.into(),
                    nickname: new_commit.nickname.into(),
                    parent_id: reply_to,
                },
                &[Skip::Column("id")],
            )
            .await?
            .last_insert_id
            .unwrap();
        let commit = tx.fetch_by_column("id", &last_id).await?;
        tx.commit().await?;
        Ok(commit)
    }

    #[inline]
    pub async fn reply_to<S1, S2, S3>(
        &self,
        rb: &Rbatis,
        new_commit: NewCommit<S1, S2, S3>,
    ) -> Result<Self, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
        S3: Into<CompactStr> + Clone,
    {
        Self::insert(rb, self.post_id, new_commit, Some(self.id))
            .await
    }

    #[inline]
    pub async fn query_replies(&self, rb: &Rbatis) -> Result<Vec<Self>, Error> {
        rb.fetch_list_by_wrapper(&rb.new_wrapper().eq("parent_id", self.id)).await.map_err(Into::into)
    }
}
