#[cfg(test)]
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
