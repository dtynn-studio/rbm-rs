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

    #[allow(non_camel_case_types)]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy)]
    pub enum Sound {
        SOUND_ID_ATTACK = 0x101,
        SOUND_ID_SHOOT = 0x102,
        SOUND_ID_SCANNING = 0x103,
        SOUND_ID_RECOGNIZED = 0x104,
        SOUND_ID_GIMBAL_MOVE = 0x105,
        SOUND_ID_COUNT_DOWN = 0x106,

        SOUND_ID_1C = 0x107,
        SOUND_ID_1C_SHARP = 0x108,
        SOUND_ID_1D = 0x109,
        SOUND_ID_1D_SHARP = 0x10A,
        SOUND_ID_1E = 0x10B,
        SOUND_ID_1F = 0x10C,
        SOUND_ID_1F_SHARP = 0x10D,
        SOUND_ID_1G = 0x10e,
        SOUND_ID_1A = 0x110,
        SOUND_ID_1A_SHARP = 0x111,
        SOUND_ID_1B = 0x112,
        SOUND_ID_2C = 0x113,
        SOUND_ID_2C_SHARP = 0x114,
        SOUND_ID_2D = 0x115,
        SOUND_ID_2D_SHARP = 0x116,
        SOUND_ID_2E = 0x117,
        SOUND_ID_2F = 0x118,
        SOUND_ID_2F_SHARP = 0x119,
        SOUND_ID_2G = 0x11A,
        SOUND_ID_2G_SHARP = 0x11B,
        SOUND_ID_2A = 0x11C,
        SOUND_ID_2A_SHARP = 0x11D,
        SOUND_ID_2B = 0x11E,
        SOUND_ID_3C = 0x11F,
        SOUND_ID_3C_SHARP = 0x120,
        SOUND_ID_3D = 0x121,
        SOUND_ID_3D_SHARP = 0x122,
        SOUND_ID_3E = 0x123,
        SOUND_ID_3F = 0x124,
        SOUND_ID_3F_SHARP = 0x125,
        SOUND_ID_3G = 0x126,
        SOUND_ID_3G_SHARP = 0x127,
        SOUND_ID_3A = 0x128,
        SOUND_ID_3A_SHARP = 0x129,
        SOUND_ID_3B = 0x12A,
    }
}
