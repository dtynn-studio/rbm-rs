use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, V1},
        Serialize,
    },
    util::algo::{crc16_calc, crc8_calc},
    Result,
};

impl_v1_cmd!(AIInit, (), CMD_SET_CTRL, 0xe9);

#[derive(Debug)]
pub struct AIInit {
    pub addr: u16,
    pub sender: u16,
    pub receiver: u16,
    pub cmd: u16,
    pub seq_num: u16,
    pub len: u16,
    pub attr: u8,
}

impl Default for AIInit {
    fn default() -> Self {
        AIInit {
            addr: 0x0103,
            sender: 0x0103,
            receiver: 0x0301,
            cmd: 0x020d,
            seq_num: 0,
            len: 2,
            attr: 0,
        }
    }
}

#[inline]
fn split_u16(n: u16) -> (u8, u8) {
    ((n & 0xff) as u8, ((n >> 8) & 0xff) as u8)
}

impl Serialize<V1> for AIInit {
    const SIZE_HINT: usize = 17;

    fn ser(&self, w: &mut impl std::io::Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE_HINT];

        buf[0] = 0xAA;
        (buf[1], buf[2]) = split_u16(self.len);
        let crc_h = crc8_calc(&buf[0..3], Some(0x11));
        buf[3] = crc_h;
        (buf[4], buf[5]) = split_u16(self.sender);
        (buf[6], buf[7]) = split_u16(self.receiver);
        buf[8] = self.attr;
        (buf[9], buf[10]) = split_u16(self.seq_num);
        (buf[11], buf[12]) = split_u16(self.cmd);
        (buf[13], buf[14]) = split_u16(self.addr);

        let crc_h16 = crc16_calc(&buf[0..Self::SIZE_HINT - 2], Some(0x4F19));
        (buf[15], buf[16]) = split_u16(crc_h16);

        w.write_all(&buf).map_err(From::from)
    }
}
