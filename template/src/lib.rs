#![feature(result_flattening)]
#![feature(box_syntax)]

use anyhow::Context;
use handlebars::Handlebars;
use serde::Serialize;

use crate::helpers::{newline_helper, nothing, render, truncate};
use crate::template_provider::{
    EmbedTemplateProvider, LocalFilesProvider, TemplateProvider,
};

mod helpers;
mod template_provider;

pub struct TemplateManager<'reg> {
    hbs: Handlebars<'reg>,
    provider: TemplateProvider,
}

impl<'reg> TemplateManager<'reg> {
    pub fn new() -> anyhow::Result<Self> {
        config::init(vec![]).unwrap();
        let config_guard = config::get_config_temp();
        let config = config_guard.render();
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(*config.strict_mode());
        hbs.set_dev_mode(*config.dev_mode());
        hbs.register_escape_fn(str::to_owned);
        hbs.register_helper("newline", box newline_helper);
        hbs.register_helper("pass", box nothing);
        hbs.register_helper("render", box render);
        hbs.register_helper("truncate", box truncate);

        let provider = if let Some(path) = config.template() {
            TemplateProvider::new(LocalFilesProvider(path.clone()))
        } else {
            TemplateProvider::new(EmbedTemplateProvider)
        };
        provider
            .load_all(&mut hbs)
            .context("load template provider")?;
        Ok(TemplateManager { hbs, provider })
    }

    pub fn render<S, D>(
        &self,
        name: S,
        data: &D,
    ) -> anyhow::Result<String>
    where
        S: AsRef<str>,
        D: Serialize,
    {
        self.hbs
            .render(name.as_ref(), data)
            .context("render template")
    }

    pub fn hbs(&self) -> &Handlebars<'reg> {
        &self.hbs
    }

    pub fn provider(&self) -> &TemplateProvider {
        &self.provider
    }
}

#[derive(Serialize)]
struct A;
#[tokio::test]
async fn render_test() {
    let tg = TemplateManager::new().unwrap();
    dbg!(tg.hbs.get_templates());
}
