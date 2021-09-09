#![feature(result_flattening)]
#![feature(box_syntax)]

use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde::Serialize;

use crate::template_provider::{
    EmbedTemplateProvider, LocalFilesProvider, Provider,
    TemplateProvider,
};

mod template_provider;

pub struct TemplateManager<'reg> {
    hbs: Handlebars<'reg>,
    provider: TemplateProvider,
}

impl<'reg> TemplateManager<'reg> {
    pub fn new(path: Option<PathBuf>) -> Self {
        TemplateManager {
            hbs: {
                let mut hbs = Handlebars::new();
                hbs.set_strict_mode(true);
                #[cfg(debug_assertions)]
                hbs.set_dev_mode(true);
                hbs
            },
            provider: match path {
                None => TemplateProvider::new(EmbedTemplateProvider),
                Some(path) => {
                    TemplateProvider::new(LocalFilesProvider(path))
                }
            },
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        for (data, name) in &self.provider.get_all() {
            self.hbs.register_template_string(
                name,
                std::str::from_utf8(data).unwrap(),
            )?;
        }

        Ok(())
    }

    pub fn render<S, D>(
        &self,
        name: S,
        data: &D,
    ) -> Result<String, handlebars::RenderError>
    where
        S: AsRef<str>,
        D: Serialize,
    {
        self.hbs.render(name.as_ref(), data)
    }

    pub fn hbs(&self) -> &Handlebars<'reg> {
        &self.hbs
    }
}

#[derive(Serialize)]
struct A;
#[tokio::test]
async fn render_test() {
    let mut tg = TemplateManager::new(None);
    tg.load().unwrap();
    dbg!(tg.hbs.get_templates());
    dbg!(tg.render("index", &A));
}
