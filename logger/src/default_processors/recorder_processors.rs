use std::sync::Arc;

use async_trait::async_trait;

use crate::{Processor, Record};

pub struct RecorderProcessors;

impl Default for RecorderProcessors {
    fn default() -> Self {
        RecorderProcessors
    }
}

#[async_trait]
impl Processor for RecorderProcessors {
    type Output = anyhow::Result<()>;

    async fn process(&mut self, record: Arc<Record>) -> Self::Output {
        let size = crate::default_processors::format(record).len();
        states::STATES.log_count.fetch_add(1, states::ORDER);
        states::STATES.log_size.fetch_add(size as u64, states::ORDER);
        Ok(())
    }
}
