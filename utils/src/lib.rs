pub use simple_i18n::{i18n, lang};
use std::str::FromStr;

pub mod ser_de;
pub mod unit;
pub mod password_hash;

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
macro_rules! any_try {
    ($func:expr, $format:literal, $($arg:expr),+) => {
        {
            use anyhow::Context;
            $func.with_context(|| {
                format!(
                    $format,
                    $($arg,)+
                )
            })
        }
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

pub trait EraseOk<E> {
    fn erase(self) -> Result<(), E>;
}

impl<T, E> EraseOk<E> for Result<T, E> {
    fn erase(self) -> Result<(), E> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub trait Consume: Sized {
    fn consume(self) {
        ()
    }
}

impl<T> Consume for T {}
