use once_cell::sync::Lazy;
pub use simple_i18n::{i18n, lang};
use crate::notify::Notify;

pub mod markdown;
pub mod password_hash;
pub mod unit;
pub mod notify;

pub static SHUTDOWN_NOTIFY: Lazy<Notify> = Lazy::new(|| Notify::new());

#[macro_export]
macro_rules! builder {
    ($name:tt $(<$($lt:lifetime),*>)?, { $($(#[$attr:meta])* $field:ident: $r#type:ty),* }) => {
        #[derive(Default)]
        pub struct $name $(<$($lt),*>)? {
            $($(#[$attr])* $field: $r#type),*
        }
        impl$(<$($lt),*>)? $name $(<$($lt),*>)? {
            $(#[allow(dead_code)] pub fn $field(mut self, $field: $r#type) -> Self {
                self.$field = $field;
                self
            })*
        }
    };
}

#[macro_export]
macro_rules! defer {
    ($func:expr) => {
        struct Defer<F>(F)
        where
            F: FnMut();

        impl<F: FnMut()> Drop for Defer<F> {
            fn drop(&mut self) {
                (self.0)();
            }
        }

        let __defer = Defer($func);
    };
}

#[macro_export]
macro_rules! pub_mods {
    ($($mods:ident),*) => {
        $(
            pub mod $mods;
        )*
    };
    (pub_use; $($mods:ident),*) => {
        $(
            mod $mods;
            pub use $mods::*;
        )*
    };
}

#[macro_export]
macro_rules! strum {
    ($name:ident, {$($member:ident),*}) => {
        use proc_macro::{stringify_lower_case, stringify_upper_case};
        impl $name {
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

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify_lower_case!($member) => Ok(Self::$member),
                    )*
                    _ => Err(())
                }
            }
        }
    };
}
