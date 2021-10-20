use anyhow::Context;
use htmlescape::encode_minimal_w;
use pulldown_cmark::{html, Options, Parser};

pub fn render(s: &str) -> anyhow::Result<String> {
    let parser = Parser::new_ext(s, Options::all());
    let mut output = Vec::with_capacity(s.len());
    html::write_html(&mut output, parser)
        .context("render markdown")?;
    Ok(unsafe { String::from_utf8_unchecked(output) })
}

pub fn render_safe(s: &str) -> anyhow::Result<String> {
    let mut output = Vec::with_capacity(s.len());
    encode_minimal_w(s, &mut output).context("escape html")?;
    let parser = Parser::new_ext(
        unsafe { std::str::from_utf8_unchecked(&output) },
        Options::all(),
    );
    let mut output = Vec::with_capacity(output.len());
    html::write_html(&mut output, parser)
        .context("render markdown")?;
    Ok(unsafe { String::from_utf8_unchecked(output) })
}
