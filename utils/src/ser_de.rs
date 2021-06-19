use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use sled::IVec;

pub trait Se: Serialize {
    fn ser(&self) -> anyhow::Result<Vec<u8>>;
}

impl<T> Se for T
    where
        T: Serialize,
{
    fn ser(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

pub trait De<'a>: Deserialize<'a> {
    fn deser<B>(data: B) -> anyhow::Result<Self>
        where
            B: AsRef<[u8]>;
}

impl<'a, T> De<'a> for T
    where
        T: DeserializeOwned,
{
    fn deser<B>(data: B) -> anyhow::Result<Self>
        where
            B: AsRef<[u8]>,
    {
        Ok(bincode::deserialize(data.as_ref())?)
    }
}

pub trait DeFromIVec: Debug + Clone + Default {
    fn de_from_ivec<'a, T>(&'a self) -> anyhow::Result<T>
        where
            T: De<'a>;
}

impl DeFromIVec for IVec {
    fn de_from_ivec<'a, T>(&'a self) -> anyhow::Result<T>
        where
            T: De<'a>,
    {
        T::deser(self)
    }
}
