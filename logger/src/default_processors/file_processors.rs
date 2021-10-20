use std::fs::create_dir_all;
use std::path::{PathBuf, Path};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::default_processors::format;
use crate::{Processor, Record};
use std::env::current_dir;

pub struct FileProcessors {
    file: Option<File>,
}

impl Default for FileProcessors {
    fn default() -> Self {
        FileProcessors { file: None }
    }
}

impl FileProcessors {
    async fn open_file() -> tokio::io::Result<File> {
        let dir = current_dir()
            .unwrap_or_else(|_| PathBuf::from("./"))
            .join("logs");
        if !dir.exists() {
            create_dir_all(&dir).ok();
        }
        OpenOptions::default()
            .write(true)
            .append(true)
            .create(true)
            .open(dir.join(Self::file_name()))
            .await
    }

    async fn open_file_and_repeat_processing(&mut self, record: Arc<Record>) -> anyhow::Result<()> {
        self.file = Some(Self::open_file().await?);
        self.process(record).await?;
        Ok(())
    }

    fn file_name() -> String {
        format!("{}.log", chrono::Local::today().format("%Y-%m-%d"))
    }
}

impl Drop for FileProcessors {
    fn drop(&mut self) {
        if let Some(Ok(file)) = self.file.take().map(|f| f.try_into_std()) {
            file.sync_all().ok();
        }
    }
}


#[async_trait]
impl Processor for FileProcessors {
    type Output = anyhow::Result<()>;

    async fn process(&mut self, record: Arc<Record>) -> Self::Output {
        if let Some(file) = &mut self.file {
            file.write_all(
                colored::Colorize::clear(&*format(record)).as_bytes(),
            )
            .await?;
            //file.sync_data().await?;
        } else {
            self.open_file_and_repeat_processing(record).await?;
        }

        Ok(())
    }
}
