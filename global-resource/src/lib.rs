use std::time::Duration;

use once_cell::sync::Lazy;

use timer::Wheel;
use utils::notify::Notify;

pub static SHUTDOWN_NOTIFY: Lazy<Notify> =
    Lazy::new(Default::default);

pub static TIME_WHEEL: Lazy<Wheel> = Lazy::new(|| {
    // 第一级轮, 10ms转动一次, 一圈100ms
    Wheel::new(10)
        .granularity(Duration::from_millis(10))
        .next_wheel(
            // 第二级轮, 100ms转动一次, 一圈1s
            Wheel::new(10)
                .granularity(Duration::from_millis(100))
                .next_wheel(
                    // 第三级轮, 1s转动一次, 一圈1min
                    Wheel::new(60)
                        .granularity(Duration::from_millis(1000))
                        .next_wheel(
                            // 第四级轮, 1min转动一次, 一圈1h
                            Wheel::new(60)
                                .granularity(Duration::from_secs(60))
                                .next_wheel(
                                    // 第五级轮, 1h转动一次, 一圈1day
                                    Wheel::new(24).granularity(
                                        Duration::from_secs(3600),
                                    ),
                                ),
                        ),
                ),
        )
});
