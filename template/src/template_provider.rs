use std::borrow::Cow;
use std::path::{Path, PathBuf};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(rust_embed::RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/small"]
struct DefaultTemplate;

pub enum Provider {
    Fs(PathBuf),
    Default,
}

impl Provider {
    pub fn new_fs<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Provider::Fs(path.as_ref().to_path_buf())
    }

    pub async fn get_content(
        &self,
        path: &Cow<'static, str>,
    ) -> anyhow::Result<Cow<'static, [u8]>> {
        Ok(match self {
            Provider::Fs(_) => {
                let mut data = Vec::new();
                File::open(path.as_ref())
                    .await?
                    .read_to_end(&mut data)
                    .await?;
                Cow::Owned(data)
            }
            Provider::Default => DefaultTemplate::get(path)
                .ok_or_else(|| {
                    anyhow::anyhow!("Unexpected embed template error")
                })?,
        })
    }

    pub fn get_hbs(&self) -> anyhow::Result<Vec<Cow<'static, str>>> {
        Ok(match self {
            Provider::Fs(path) => path
                .read_dir()?
                .filter_map(Result::ok)
                .filter(|dir_entry| {
                    let pb: PathBuf = dir_entry.path();
                    pb.extension()
                        .map(|s| s == ".hbs")
                        .unwrap_or(false)
                })
                .filter_map(|pb| {
                    pb.path()
                        .to_str()
                        .map(|s| Cow::Owned(s.to_owned()))
                })
                .collect(),
            Provider::Default => DefaultTemplate::iter().collect(),
        })
    }
}

impl Default for Provider {
    fn default() -> Self {
        Provider::Default
    }
}
