use chrono::NaiveDateTime;
use compact_str::CompactStr;
use rbatis::core::value::DateTimeNow;
use rbatis::crud::CRUDMut;
use rbatis::crud::{Skip, CRUD};
use rbatis::crud_table;
use rbatis::rbatis::Rbatis;

use error::Error;

use crate::models::comment::{Comments, NewComment};

#[derive(Default, Clone)]
pub struct NewPost<S1, S2>
where
    S1: Into<CompactStr> + Clone,
    S2: Into<CompactStr> + Clone,
{
    pub title: S1,
    pub content: S2,
}

#[crud_table]
pub struct Posts {
    pub id: u64,

    pub title: CompactStr,
    pub content: CompactStr,

    pub create_time: NaiveDateTime,
    pub last_modified_time: NaiveDateTime,
}

impl Posts {
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

    pub async fn insert<S1, S2>(
        rb: &Rbatis,
        new_post: NewPost<S1, S2>,
    ) -> Result<Self, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
    {
        let mut tx = rb.acquire_begin().await?;

        let now_time = NaiveDateTime::now();

        let last_id = tx
            .save(
                &Posts {
                    id: 0,
                    title: new_post.title.into(),
                    content: new_post.content.into(),
                    create_time: now_time,
                    last_modified_time: now_time,
                },
                &[Skip::Column("id")],
            )
            .await?
            .last_insert_id
            .unwrap();
        let post = tx.fetch_by_column("id", &last_id).await?;
        tx.commit().await?;
        Ok(post)
    }

    pub async fn update_by_id<S1, S2>(
        rb: &Rbatis,
        id: u64,
        new_post: NewPost<S1, S2>,
    ) -> Result<NaiveDateTime, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
    {
        let w = rb.new_wrapper().eq("id", id);
        let now_time = NaiveDateTime::now();
        rb.update_by_wrapper(
            &Posts {
                id,
                title: new_post.title.into(),
                content: new_post.content.into(),
                create_time: now_time,
                last_modified_time: now_time,
            },
            w,
            &[
                Skip::Value(serde_json::Value::Null),
                Skip::Column("id"),
                Skip::Column("create_time"),
            ],
        )
        .await?;

        Ok(now_time)
    }

    pub async fn update<S1, S2>(
        &mut self,
        rb: &Rbatis,
        id: u64,
        new_post: NewPost<S1, S2>,
    ) -> Result<(), Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
    {
        let update_time =
            Self::update_by_id(rb, id, new_post.clone()).await?;
        self.title = new_post.title.into();
        self.content = new_post.content.into();
        self.last_modified_time = update_time;
        Ok(())
    }

    #[inline]
    pub async fn reply<S1, S2, S3>(
        &self,
        rb: &Rbatis,
        new_comment: NewComment<S1, S2, S3>,
        reply_to: Option<u64>,
    ) -> Result<Comments, Error>
    where
        S1: Into<CompactStr> + Clone,
        S2: Into<CompactStr> + Clone,
        S3: Into<CompactStr> + Clone,
    {
        Comments::insert(rb, self.id, new_comment, reply_to).await
    }

    #[inline]
    pub async fn query_comments(
        &self,
        rb: &Rbatis,
    ) -> Result<Vec<Comments>, Error> {
        rb.fetch_list_by_wrapper(
            rb.new_wrapper().eq("post_id", self.id),
        )
        .await
        .map_err(Into::into)
    }
}
