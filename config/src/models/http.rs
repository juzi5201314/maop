crate::gen_config!(HttpConfig, {
    bind: String,
    port: u16,
    r#type: String,
    #[serde(default)]
    #[serde(deserialize_with = "utils::password_hash::password_hash")]
    password: Option<String>
});
