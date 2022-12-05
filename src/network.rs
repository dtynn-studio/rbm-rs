#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NetworkType {
    Ap = 0,
    Sta = 1,
    Rndis = 2,
}

impl Default for NetworkType {
    fn default() -> Self {
        Self::Ap
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ConnectionType {
    Udp = 0,
    Tcp = 1,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Udp
    }
}
