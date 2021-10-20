use std::borrow::Cow;
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;

use error::Error;

#[derive(rust_embed::RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/small"]
pub struct EmbedTemplateProvider;

pub struct LocalFilesProvider(pub PathBuf);

#[async_trait::async_trait]
pub trait Provider {
    fn load_all<'reg>(
        &self,
        hbs: &mut Handlebars<'reg>,
    ) -> Result<(), error::Error>;

    async fn get(
        &self,
        path: &str,
    ) -> Result<Option<Cow<'static, [u8]>>, Error>;
}

#[async_trait::async_trait]
impl Provider for EmbedTemplateProvider {
    fn load_all<'reg>(
        &self,
        hbs: &mut Handlebars<'reg>,
    ) -> Result<(), Error> {
        Self::iter().try_for_each(|path| {
            let file_name = Path::new(&*path)
                .file_name()
                .unwrap()
                .to_string_lossy();
            if file_name.as_ref().ends_with(".hbs") {
                hbs.register_template_string(
                    file_name.as_ref().trim_end_matches(".hbs"),
                    std::str::from_utf8(
                        Self::get(&*path).unwrap().data.as_ref(),
                    )
                    .unwrap(),
                )?;
            }
            Ok(())
        })
    }

    async fn get(
        &self,
        path: &str,
    ) -> Result<Option<Cow<'static, [u8]>>, Error> {
        Ok(Self::get(path).map(|file| file.data))
    }
}

#[async_trait::async_trait]
impl Provider for LocalFilesProvider {
    fn load_all<'reg>(
        &self,
        hbs: &mut Handlebars<'reg>,
    ) -> Result<(), Error> {
        WalkDir::new(&self.0)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().ends_with(".hbs"))
            .try_for_each(|e| {
                let file_name = e.file_name().to_string_lossy();
                let body = read_to_string(e.path())?;
                hbs.register_template_string(
                    file_name.as_ref(),
                    body,
                )?;
                Ok(())
            })
    }

    async fn get(
        &self,
        path: &str,
    ) -> Result<Option<Cow<'static, [u8]>>, Error> {
        let path = self.0.join(path);
        let file = OpenOptions::new().read(true).open(path).await;
        match file {
            Ok(mut file) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data).await?;
                Ok(Some(Cow::Owned(data)))
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Ok(None),
                _ => return Err(err.into()),
            },
        }
    }
}

pub struct TemplateProvider(pub Box<dyn Provider + Sync + Send>);

impl TemplateProvider {
    pub fn new<P>(provider: P) -> Self
    where
        P: Provider + Sync + Send + 'static,
    {
        TemplateProvider(box provider)
    }
}

impl Deref for TemplateProvider {
    type Target = Box<dyn Provider + Sync + Send>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
