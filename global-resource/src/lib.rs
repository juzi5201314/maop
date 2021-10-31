use once_cell::sync::Lazy;

use utils::notify::Notify;

pub static SHUTDOWN_NOTIFY: Lazy<Notify> =
    Lazy::new(Default::default);
