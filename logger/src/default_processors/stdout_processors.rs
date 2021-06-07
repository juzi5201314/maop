use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::default_processors::format;
use crate::{Processor, Record};

pub struct StdoutProcessors {
    stdout: tokio::io::Stdout,
}

impl Default for StdoutProcessors {
    fn default() -> Self {
        StdoutProcessors {
            stdout: tokio::io::stdout(),
        }
    }
}

#[async_trait]
impl Processor for StdoutProcessors {
    type Output = anyhow::Result<()>;

    async fn process(&mut self, record: Arc<Record>) -> Self::Output {
        self.stdout.write_all(format(record).as_bytes()).await?;
        Ok(())
    }
}
