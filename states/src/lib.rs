
use std::sync::atomic::Ordering;

use once_cell::sync::Lazy;

use atomic::Atomic;

static STATES: Lazy<States> = Lazy::new(States::new);

pub const ORDER: Ordering = Ordering::SeqCst;

#[derive(Default)]
pub struct States {
    log_count: Atomic<u64>,
}

impl States {
    pub fn new() -> Self {
        Default::default()
    }
}

#[test]
fn state_test() {
    let a = 12;
    STATES.log_count.store(a, ORDER);

    dbg!(STATES.log_count.load(ORDER));
}
