#![feature(box_syntax)]

use std::fmt::{Debug, Formatter};
use std::time::Duration;

use futures::future::BoxFuture;
use tokio::time::MissedTickBehavior;

#[derive(Debug, Copy, Clone)]
pub enum Follow {
    Cancel,
    Change(Duration),
    Done,
}

#[derive(Debug)]
pub struct Task<'a> {
    ty: TaskType<'a>,
    time: Duration,
    to_next: bool,
}

enum TaskType<'a> {
    Delay(BoxFuture<'a, ()>),
    Interval(Box<dyn Fn() -> BoxFuture<'a, Follow> + 'a>),
}

#[derive(Default, Debug)]
pub struct Wheel<'a> {
    slots: Vec<Slot<'a>>,
    capacity: usize,
    index: usize,
    granularity: Duration,
    next_wheel: Option<Box<Wheel<'a>>>,
    missed_tick_behavior: MissedTickBehavior,
}

#[derive(Debug)]
pub struct TaskWrapper<'a> {
    task: Task<'a>,
    round: usize,
}

#[derive(Default, Debug)]
pub struct Slot<'a> {
    tasks: Vec<TaskWrapper<'a>>,
}

impl<'a> Wheel<'a> {
    pub fn new(capacity: usize) -> Wheel<'a> {
        let slots =
            (0..capacity).map(|_| Default::default()).collect();
        Wheel {
            slots,
            capacity,
            missed_tick_behavior: MissedTickBehavior::Burst,
            ..Default::default()
        }
    }

    pub fn granularity(mut self, granularity: Duration) -> Self {
        self.granularity = granularity;
        self
    }

    pub fn missed_tick_behavior(
        mut self,
        missed_tick_behavior: MissedTickBehavior,
    ) -> Self {
        self.missed_tick_behavior = missed_tick_behavior;
        self
    }

    pub fn next_wheel(mut self, next_wheel: Wheel<'a>) -> Self {
        self.next_wheel = Some(box next_wheel);
        self
    }

    pub fn add_task(&mut self, mut task: Task<'a>) {
        let slot_count = (task.time.as_millis()
            / self.granularity.as_millis())
            as usize;

        let (mut slot_index, round) = if slot_count > self.capacity {
            if let Some(wheel) = &self
                .next_wheel
                .as_ref()
                .filter(|wheel| wheel.granularity <= task.time)
            {
                let a = (task.time.as_millis()
                    % wheel.granularity.as_millis())
                    as u64;
                task.time -= Duration::from_millis(a);
                task.to_next = true;
                let slot_count = (a as u128
                    / self.granularity.as_millis())
                    as usize;
                (slot_count % self.capacity, 0)
            } else {
                let round = slot_count / self.capacity;
                let mut slot_index = slot_count % self.capacity;
                if slot_index == 0 {
                    slot_index += 1;
                }

                (slot_index + self.index, round)
            }
        } else if slot_count == 0 {
            (self.index + 1, 0)
        } else {
            let slot_index = self.index + slot_count;
            (slot_index, 0)
        };

        if slot_index >= self.capacity {
            slot_index -= self.capacity
        }

        let slot: &mut Slot = unsafe {
            self.slots.get_unchecked_mut(slot_index as usize)
        };
        slot.tasks.push(TaskWrapper { task, round });
    }

    #[async_recursion::async_recursion(?Send)]
    pub async fn roll(&mut self) {
        let roll_next = self.next_index();
        self.do_tasks().await;
        if roll_next {
            if let Some(wheel) = &mut self.next_wheel {
                wheel.roll().await;
            }
        }
    }

    async fn do_tasks(&mut self) {
        let tasks = &mut self.slots[self.index].tasks;
        if tasks.is_empty() {
            return;
        }

        let mut no_run_tasks = Vec::new();
        let mut interval_tasks = Vec::new();

        while let Some(mut wrapper) = tasks.pop() {
            if wrapper.round > 0 {
                wrapper.round -= 1;
                no_run_tasks.push(wrapper);
            } else if wrapper.task.to_next {
                wrapper.task.to_next = false;
                self.next_wheel
                    .as_mut()
                    .unwrap()
                    .add_task(wrapper.task);
            } else {
                match wrapper.task.ty {
                    TaskType::Delay(fut) => {
                        fut.await;
                    }
                    TaskType::Interval(ref f) => {
                        let fut = f();
                        let follow = fut.await;

                        match follow {
                            Follow::Done => {
                                interval_tasks.push(wrapper.task);
                            }
                            Follow::Cancel => {}
                            Follow::Change(time) => {
                                wrapper.task.time = time;
                                interval_tasks.push(wrapper.task);
                            }
                        }
                    }
                }
            }
        }
        tasks.extend(no_run_tasks);
        interval_tasks
            .into_iter()
            .for_each(|task| self.add_task(task));
    }

    fn next_index(&mut self) -> bool {
        if self.index == self.capacity - 1 {
            self.index = 0;
            true
        } else {
            self.index += 1;
            false
        }
    }

    pub async fn run(mut self) {
        let mut timer = tokio::time::interval(self.granularity);
        timer.set_missed_tick_behavior(self.missed_tick_behavior);
        timer.tick().await;
        loop {
            timer.tick().await;
            self.roll().await;
        }
    }
}

impl<'a> Task<'a> {
    pub fn delay(fut: BoxFuture<'a, ()>, time: Duration) -> Self {
        Task {
            ty: TaskType::Delay(fut),
            time,
            to_next: false,
        }
    }

    pub fn interval<F>(func: F, time: Duration) -> Self
    where
        F: Fn() -> BoxFuture<'a, Follow> + 'a,
    {
        Task {
            ty: TaskType::Interval(box func),
            time,
            to_next: false,
        }
    }
}

impl<'a> Debug for TaskType<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TaskType::Delay(_) => "Delay",
                TaskType::Interval(_) => "Interval",
            }
        )
    }
}

#[cfg(test)]
macro_rules! accuracy_test_in_situ_m {
    ($time:expr, $count:expr) => {
        Task::interval(
            move || {
                async move {
                    static COUNT: AtomicCell<usize> =
                        AtomicCell::new(0);
                    static LAST: Lazy<AtomicCell<Instant>> =
                        Lazy::new(|| AtomicCell::new(Instant::now()));
                    let actual = LAST.swap(Instant::now()).elapsed();

                    if actual > $time {
                        let error = actual - $time;
                        println!(
                            "in_situ-{:?}, {:?} error",
                            $time, error
                        );
                        if COUNT.fetch_add(1) >= $count {
                            Follow::Cancel
                        } else {
                            Follow::Done
                        }
                    } else {
                        Follow::Done
                    }
                }
                .boxed()
            },
            $time,
        )
    };
}

#[tokio::test]
async fn test() {
    use crossbeam_utils::atomic::AtomicCell;
    use futures::FutureExt;
    use once_cell::sync::Lazy;

    use std::time::Instant;

    let mut wheel = Wheel::new(10)
        .granularity(Duration::from_millis(10))
        .next_wheel(
            Wheel::new(10)
                .granularity(Duration::from_millis(100))
                .next_wheel(
                    Wheel::new(60)
                        .granularity(Duration::from_millis(1000)),
                ),
        );

    wheel.add_task(accuracy_test_in_situ_m!(
        Duration::from_millis(50),
        5
    ));
    wheel.add_task(accuracy_test_in_situ_m!(
        Duration::from_millis(500),
        5
    ));
    wheel.add_task(accuracy_test_in_situ_m!(
        Duration::from_millis(1000),
        5
    ));
    wheel.add_task(accuracy_test_in_situ_m!(
        Duration::from_millis(10000),
        5
    ));

    wheel.run().await;
}
