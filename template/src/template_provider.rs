use std::borrow::Cow;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use walkdir::WalkDir;

#[derive(rust_embed::RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/small"]
pub struct EmbedTemplateProvider;

pub struct LocalFilesProvider(pub PathBuf);

pub trait Provider {
    fn get_all(&self)
        -> Vec<(Cow<'static, [u8]>, Cow<'static, str>)>;
}

impl Provider for EmbedTemplateProvider {
    fn get_all(
        &self,
    ) -> Vec<(Cow<'static, [u8]>, Cow<'static, str>)> {
        Self::iter()
            .map(|path| {
                (
                    Self::get(&*path).unwrap().data,
                    Cow::Owned(
                        Path::new(&*path)
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .trim_end_matches(".hbs")
                            .to_owned(),
                    ),
                )
            })
            .collect()
    }
}

impl Provider for LocalFilesProvider {
    fn get_all(
        &self,
    ) -> Vec<(Cow<'static, [u8]>, Cow<'static, str>)> {
        WalkDir::new(&self.0)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().ends_with(".hbs"))
            .map(|e| {
                File::open(e.path()).map(move |mut file| {
                    let mut data = Vec::with_capacity(
                        file.metadata()
                            .map(|meta| meta.len())
                            .unwrap_or_default()
                            as usize,
                    );
                    file.read_to_end(&mut data)?;
                    Ok((
                        Cow::Owned(data),
                        Cow::Owned(
                            e.file_name()
                                .to_string_lossy()
                                .trim_end_matches(".hbs")
                                .to_owned(),
                        ),
                    ))
                })
            })
            .filter_map(|res| {
                res.flatten()
                    .map_err(|err| {
                        log::warn!(
                            "failed to read template file. {:?}",
                            err
                        );
                        err
                    })
                    .ok()
            })
            .collect()
    }
}

pub struct TemplateProvider(pub Box<dyn Provider>);

impl TemplateProvider {
    pub fn new<P>(provider: P) -> Self
    where
        P: Provider + 'static,
    {
        TemplateProvider(box provider)
    }
}

impl Deref for TemplateProvider {
    type Target = Box<dyn Provider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
