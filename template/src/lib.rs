use std::path::Path;

use handlebars::Handlebars;
use serde::Serialize;

use crate::template_provider::Provider;

mod template_provider;

pub struct TemplateGroup<'reg> {
    hbs: Handlebars<'reg>,
    provider: Provider,
}

impl<'reg> TemplateGroup<'reg> {
    pub fn new<P>(path: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        TemplateGroup {
            hbs: {
                let mut hbs = Handlebars::new();
                hbs.set_strict_mode(true);
                hbs
            },
            provider: path
                .map(|path| Provider::new_fs(path))
                .unwrap_or_default(),
        }
    }

    pub async fn load(&mut self) -> anyhow::Result<()> {
        for path in self.provider.get_hbs()?.iter() {
            if path.ends_with(".hbs") {
                let data = self.provider.get_content(&path).await?;
                self.hbs.register_template_string(
                    path.trim_end_matches(".hbs"),
                    std::str::from_utf8(&data)?,
                )?;
            }
        }

        Ok(())
    }

    pub fn render<S, D>(&self, name: S, data: &D) -> Result<String, handlebars::RenderError> where S: AsRef<str>, D: Serialize {
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
    let mut tg = TemplateGroup::new::<String>(None);
    tg.load().await.unwrap();
    dbg!(tg.hbs.get_templates());
    dbg!(tg.render("index", &A));
}

// TODO: jekyll compatible
/*#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn template_test() {
        use liquid::partials::{InMemorySource, LazyCompiler};
        let template = liquid::ParserBuilder::with_stdlib()
            .tag(liquid_lib::jekyll::IncludeTag)
            .partials(LazyCompiler::new({
                let mut source = InMemorySource::new();
                source.add(
                    "head.html",
                    include_str!(
                        "../tests/monophase/_includes/head.html"
                    ),
                );
                source
            }))
            .build()
            .unwrap()
            //.parse("{% include \"head.html\" %}")
            .parse(include_str!(
                "../tests/monophase/_layouts/default.html"
            ))
            .unwrap();

        let globals = liquid::object!({
            "page": liquid::object!({
                "lang": "zh"
            }),
            "site": liquid::object!({
                "lang": "zh"
            }),
        });

        let output = template.render(&globals).unwrap();
        dbg!(output);
    }
}
*/
