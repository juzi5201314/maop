pub use simple_i18n::{i18n, lang};

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
