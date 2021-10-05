use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RenderStrategy {
    SSR,
    CSR,
}

crate::gen_config!(RenderConfig, { default_render: RenderStrategy });
