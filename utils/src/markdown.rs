use anyhow::Context;
use pulldown_cmark::{html, Options, Parser};

pub fn render(s: &str) -> anyhow::Result<String> {
    let parser = Parser::new_ext(s, Options::all());
    let mut output = Vec::with_capacity(s.len());
    html::write_html(&mut output, parser)
        .context("render markdown")?;
    Ok(unsafe { String::from_utf8_unchecked(output) })
}

pub fn render_safe(s: &str) -> anyhow::Result<String> {
    let parser = Parser::new_ext(s, Options::all());
    let mut output = Vec::with_capacity(s.len() * 2);
    html::write_html(&mut output, parser)
        .context("render markdown")?;
    Ok(ammonia::clean(unsafe {
        std::str::from_utf8_unchecked(&output)
    }))
}

#[inline]
pub fn html_escape(s: &str) -> String {
    ammonia::clean(s)
}
