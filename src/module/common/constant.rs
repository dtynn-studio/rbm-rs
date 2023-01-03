pub mod v1 {
    use crate::{proto::v1::Receiver, util::host2byte};
    pub const DEFAULT_TARGET: Option<Receiver> = Some(host2byte(9, 0));

    #[repr(u64)]
    pub enum Uid {
        Battery = 0x000200096862229f,
        GimbalBase = 0x00020009f5882874,
        Velocity = 0x0002000949a4009c,
        Esc = 0x00020009c14cb7c5,
        Attitude = 0x000200096b986306,
        Imu = 0x00020009a7985b8d,
        Position = 0x00020009eeb7cece,
        SaStatus = 0x000200094a2c6d55,
        ChassisMode = 0x000200094fcb1146,
        Sbus = 0x0002000988223568,
        Servo = 0x000200095f0059e7,
        Arm = 0x0002000926abd64d,
        Gripper = 0x00020009124d156a,
        GimbalPos = 0x00020009f79b3c97,
        Stick = 0x0002000955e9a0fa,
        MoveMode = 0x00020009784c7bfd,
        Tof = 0x0002000986e4c05a,
        Pinboard = 0x00020009eebb9ffc,
    }
}
