use std::net::IpAddr;
use std::collections::HashMap;
use utils::unit::byte_unit::ByteUnit;

crate::gen_config!(RocketConfig, {
    addr: IpAddr,
    port: u16,
    workers: Option<usize>,
    keep_alive: Option<u32>,
    limits: Option<HashMap<String, ByteUnit>>
});
