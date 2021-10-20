use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeUnit(Duration);

impl TimeUnit {
    pub fn duration(&self) -> &Duration {
        &self.0
    }
}

impl Default for TimeUnit {
    fn default() -> Self {
        TimeUnit(Duration::from_secs(0))
    }
}

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TimeUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! err {
            () => {
                crate::i18n!("errors.unit.time_unit_format")
                    .to_owned()
            };
        }
        let chars = s.chars();
        let unit = chars
            .clone()
            .filter_map(|c| c.is_ascii_alphabetic().then(|| c))
            .collect::<String>();
        let time = chars
            .filter_map(|c| c.is_digit(10).then(|| c))
            .collect::<String>()
            .parse::<u64>()
            .map_err(|_| err!())?;

        Ok(TimeUnit(match unit.as_ref() {
            "ns" => Duration::from_nanos(time),
            "us" => Duration::from_micros(time),
            "ms" => Duration::from_millis(time),
            "s" => Duration::from_secs(time),
            "m" => Duration::from_secs(time * 60),
            "h" => Duration::from_secs(time * 60 * 60),
            "d" => Duration::from_secs(time * 60 * 60 * 24),
            _ => return Err(err!()),
        }))
    }
}

impl Serialize for TimeUnit {
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

impl<'de> Deserialize<'de> for TimeUnit {
    fn deserialize<D>(deserializer: D) -> Result<TimeUnit, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimeVisitor)
    }
}

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = TimeUnit;

    fn expecting(
        &self,
        formatter: &mut Formatter<'_>,
    ) -> fmt::Result {
        formatter
            .write_str(crate::i18n!("errors.unit.time_unit_format"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TimeUnit::from_str(v).map_err(|e| E::custom(e))
    }
}
