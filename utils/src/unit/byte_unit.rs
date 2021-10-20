use std::fmt;
use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde::de::Visitor;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ByteUnit(byte_unit::Byte);

impl ByteUnit {
    pub fn get_bytes(&self) -> u64 {
        self.0.get_bytes()
    }
}

impl FromStr for ByteUnit {
    type Err = byte_unit::ByteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ByteUnit(byte_unit::Byte::from_str(s)?))
    }
}

impl fmt::Display for ByteUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


impl Serialize for ByteUnit {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for ByteUnit {
    fn deserialize<D>(deserializer: D) -> Result<ByteUnit, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ByteVisitor)
    }
}

struct ByteVisitor;

impl<'de> Visitor<'de> for ByteVisitor {
    type Value = ByteUnit;

    fn expecting(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter
            .write_str(crate::i18n!("errors.unit.byte_unit_format"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        ByteUnit::from_str(v).map_err(|e| E::custom(e))
    }
}
