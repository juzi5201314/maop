crate::gen_config!(HttpConfig, {
    bind: String,
    port: u16,
    r#type: String,
    password: Option<String>
});
