use compact_str::CompactString;
use utils::unit::time_unit::TimeUnit;

#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ListenType {
    Http,
    Uds,
}

crate::gen_config!(HttpConfig, {
    bind: CompactString,
    port: u16,
    r#type: ListenType,
    session_expiry: TimeUnit,
    overdue_check_interval: TimeUnit,
    cors: Vec<CompactString>
});
