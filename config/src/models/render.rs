use std::path::PathBuf;

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Copy, Clone,
)]
#[serde(rename_all = "lowercase")]
pub enum RenderStrategy {
    SSR,
    CSR,
}

crate::gen_config!(RenderConfig, { default_render: RenderStrategy, strict_mode: bool, dev_mode: bool, template: Option<PathBuf> });
