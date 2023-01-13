pub mod cmd;
pub mod sub;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SensorIndex {
    No1 = 1,
    No2 = 2,
    No3 = 3,
    No4 = 4,
    No5 = 5,
    No6 = 6,
    No7 = 7,
    No8 = 8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SensorPort {
    Port1 = 1,
    Port2 = 2,
}
