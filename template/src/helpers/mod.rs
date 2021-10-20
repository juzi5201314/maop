use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output,
    RenderContext,
};

#[inline]
pub fn newline_helper(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write("\n")?;
    Ok(())
}

#[inline]
pub fn nothing(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    _: &mut dyn Output,
) -> HelperResult {
    Ok(())
}

#[inline]
pub fn truncate(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let str =
        h.param(0).map(|p| p.value().render()).unwrap_or_default();
    let max = h
        .param(1)
        .map(|p| p.value().as_u64())
        .flatten()
        .unwrap_or(20);
    out.write(&match str.char_indices().nth(max as usize) {
        None => str,
        Some((idx, _)) => format!(
            "{}{}",
            &str[..idx],
            h.param(2)
                .map(|p| p.value().render())
                .as_deref()
                .unwrap_or("...")
        ),
    })?;
    Ok(())
}

#[inline]
pub fn render(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let val = h.param(0).unwrap().value().render();
    out.write(&val)?;
    Ok(())
}
