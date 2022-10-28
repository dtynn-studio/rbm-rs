pub const fn host2byte(host: u8, index: u8) -> u8 {
    index << 5 | host
}

pub const fn byte2host(b: u8) -> (u8, u8) {
    (b & 0x1f, b >> 5)
}
