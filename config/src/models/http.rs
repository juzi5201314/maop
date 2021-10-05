use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ListenType {
    Http,
    Udp,
}

crate::gen_config!(HttpConfig, {
    bind: String,
    port: u16,
    r#type: ListenType,
    #[serde(default)]
    #[serde(deserialize_with = "utils::password_hash::password_hash")]
    password: Option<String>
});
