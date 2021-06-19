use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::default_processors::format;
use crate::{Processor, Record};

pub struct StdoutProcessors {
    stdout: tokio::io::Stdout,
    flush_every_time: bool
}

impl Default for StdoutProcessors {
    fn default() -> Self {
        StdoutProcessors {
            stdout: tokio::io::stdout(),
            flush_every_time: *config::get_config().as_ref().log().flush_stdout_every_time()
        }
    }
}

#[async_trait]
impl Processor for StdoutProcessors {
    type Output = anyhow::Result<()>;

    async fn process(&mut self, record: Arc<Record>) -> Self::Output {
        self.stdout.write_all(format(record).as_bytes()).await?;
        if self.flush_every_time {
            self.stdout.flush().await?;
        }
        Ok(())
    }
}
