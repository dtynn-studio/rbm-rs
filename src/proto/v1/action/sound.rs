use crate::{
    proto::{action, cmd::Command, host2byte, v1, Completed},
    Result,
};

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RobotSound {
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

#[derive(Debug)]
pub struct PlaySoundAction {
    sound_id: RobotSound,
    play_times: u8,

    pub status: v1::V1ActionStatus,
}

impl PlaySoundAction {
    pub fn new(sound_id: RobotSound, play_times: u8) -> Self {
        PlaySoundAction {
            sound_id,
            play_times,
            status: Default::default(),
        }
    }
}

impl action::Action for PlaySoundAction {
    type Cmd = v1::ctrl::PlaySound;
    type Event = v1::ctrl::SoundPushEvent;
    type Status = v1::V1ActionStatus;

    const RECEIVER: u8 = host2byte(9, 0);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        Ok(v1::ctrl::PlaySound {
            sound_id: self.sound_id as u32,
            play_times: self.play_times,
            ..Default::default()
        })
    }

    fn is_completed(&self) -> bool {
        self.status.is_completed()
    }

    fn apply_progress(
        &mut self,
        progress: action::Progress<<Self::Cmd as Command>::Response, Self::Status, Self::Event>,
    ) -> Result<bool> {
        match progress {
            action::Progress::Response(resp) => {
                self.status.state = resp.into();
            }

            action::Progress::Event(status, _evt) => {
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
