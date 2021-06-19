pub use crate___name::crate_name;

#[macro_export]
macro_rules! _log {
    ($lvl:expr, $arg:expr) => {
        $crate::_log!($lvl, "{}", $arg);
    };

    ($lvl:expr, $($arg:tt)+) => {
        $crate::_log($lvl,
            format!($($arg)+),
            module_path!(),
            ::std::borrow::Cow::Borrowed(file!()),
            line!(),
            $crate::crate_name!()
        );
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::_log!($crate::Level::Debug, $($arg)+);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::_log!($crate::Level::Info, $($arg)+);
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {
        $crate::_log!($crate::Level::Warning, $($arg)+);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::_log!($crate::Level::Error, $($arg)+);
    };
}
