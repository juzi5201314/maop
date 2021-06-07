use proc_macro::{stringify_lower_case, stringify_upper_case};
use serde::{Deserialize, Serialize, Serializer};
use std::cmp;
use std::fmt::{Display, Formatter};

macro_rules! _strum {
    ($($member:ident),*) => {
        impl Level {
            pub fn to_str(self) -> &'static str {
                match self {
                    $(
                        Self::$member => stringify!($member),
                    )*
                }
            }

            pub fn to_uppercase_str(self) -> &'static str {
                match self {
                    $(
                        Self::$member => stringify_upper_case!($member),
                    )*
                }
            }

            pub fn to_lowercase_str(self) -> &'static str {
                match self {
                    $(
                        Self::$member => stringify_lower_case!($member),
                    )*
                }
            }
        }

        impl From<String> for Level {
            fn from(s: String) -> Self {
                match s.as_ref() {
                    $(
                        stringify_lower_case!($member) => Self::$member,
                    )*
                    _ => panic!("{}: {}", utils::i18n!("errors.logger.nonexistent_level"), &s)
                }
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Eq, Deserialize)]
pub enum Level {
    Debug = 1,
    Info,
    Warning,
    Error,
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

_strum! {
        Debug,
        Info,
        Warning,
        Error
}
