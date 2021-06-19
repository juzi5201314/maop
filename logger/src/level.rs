use serde::{Deserialize, Serialize, Serializer};
use std::cmp;
use std::fmt::{Display, Formatter};

utils::strum!(Level, {
    Debug,
    Info,
    Warning,
    Error
});

#[derive(Debug, Copy, Clone, Eq, Deserialize)]
pub enum Level {
    Debug = 1,
    Info,
    Warning,
    Error,
}

impl From<log::Level> for Level {
    fn from(lvl: log::Level) -> Self {
        match lvl {
            log::Level::Error => Level::Error,
            log::Level::Warn => Level::Warning,
            log::Level::Debug => Level::Debug,
            log::Level::Info => Level::Info,
            log::Level::Trace => Level::Debug
        }
    }
}

impl Serialize for Level {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd for Level {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }

    #[inline]
    fn lt(&self, other: &Level) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &Level) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &Level) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &Level) -> bool {
        *self as usize >= *other as usize
    }
}

impl Ord for Level {
    #[inline]
    fn cmp(&self, other: &Level) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}
