use std::{
    cmp::Ordering,
    ops::{AddAssign, DivAssign, MulAssign, SubAssign},
};

use crate::{
    util::decimal::{MaybeFrom, MaybeInto, MaybeRound},
    Error, Result,
};

pub const CHASSIS_POS_X_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-5.0),
    end: Some(5.0),
    decimal: 0,
    scale: 100.0,
    delta: 0.0,
    unit: "m",
};

pub const CHASSIS_POS_Y_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-5.0),
    end: Some(5.0),
    decimal: 0,
    scale: 100.0,
    delta: 0.0,
    unit: "m",
};

pub const CHASSIS_POS_Z_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-1800.0),
    end: Some(1800.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const CHASSIS_SPEED_XY_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(0.5),
    end: Some(2.0),
    decimal: 0,
    scale: 160.0,
    delta: -70.0,
    unit: "m/s",
};

pub const CHASSIS_SPEED_Z_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(10.0),
    end: Some(540.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°/s",
};

pub const CHASSIS_POS_X_SUB_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 5,
    scale: 1.0,
    delta: 0.0,
    unit: "",
};

pub const CHASSIS_POS_Y_SUB_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 5,
    scale: 1.0,
    delta: 0.0,
    unit: "",
};

pub const CHASSIS_POS_Z_SUB_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 2,
    scale: 10.0,
    delta: 0.0,
    unit: "",
};

pub const WHEEL_SPD_CONVERTOR: UnitConvertor<i16> = UnitConvertor {
    start: Some(-1000),
    end: Some(1000),
    decimal: 0,
    scale: 1,
    delta: 0,
    unit: "rpm",
};

pub const CHASSIS_SPD_X_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-3.5),
    end: Some(3.5),
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "m/s",
};

pub const CHASSIS_SPD_Y_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-3.5),
    end: Some(3.5),
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "m/s",
};

pub const CHASSIS_SPD_Z_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-600.0),
    end: Some(600.0),
    decimal: 0,
    scale: 1.0,
    delta: 0.0,
    unit: "°/s",
};

pub const PWM_VALUE_CONVERTOR: UnitConvertor<u16> = UnitConvertor {
    start: Some(0),
    end: Some(100),
    decimal: 0,
    scale: 1,
    delta: 0,
    unit: "%",
};

pub const PWM_FREQ_CONVERTOR: UnitConvertor<u16> = UnitConvertor {
    start: Some(0),
    end: Some(50000),
    decimal: 0,
    scale: 1,
    delta: 0,
    unit: "Hz",
};

pub const CHASSIS_YAW_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-180.0),
    end: Some(180.0),
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "°",
};

pub const CHASSIS_PITCH_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-180.0),
    end: Some(180.0),
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "°",
};

pub const CHASSIS_ROLL_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-180.0),
    end: Some(180.0),
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "°",
};

pub const CHASSIS_ACC_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 5,
    scale: 1.0,
    delta: 0.0,
    unit: "",
};

pub const CHASSIS_GYRO_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 5,
    scale: 1.0,
    delta: 0.0,
    unit: "",
};

pub const GIMBAL_PITCH_SPEED_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-540.0),
    end: Some(540.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°/s",
};

pub const GIMBAL_YAW_SPEED_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-540.0),
    end: Some(540.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°/s",
};

pub const GIMBAL_PITCH_MOVE_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-55.0),
    end: Some(55.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const GIMBAL_YAW_MOVE_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-500.0),
    end: Some(500.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const GIMBAL_PITCH_TARGET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-20.0),
    end: Some(35.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const GIMBAL_YAW_TARGET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(-250.0),
    end: Some(250.0),
    decimal: 0,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const GIMBAL_RECENTER_SPEED_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(0.0),
    end: Some(360.0),
    decimal: 0,
    scale: 1.0,
    delta: 0.0,
    unit: "°/s",
};

pub const GIMBAL_PITCH_MOVE_SPEED_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(0.0),
    end: Some(540.0),
    decimal: 0,
    scale: 1.0,
    delta: 0.0,
    unit: "°/s",
};

pub const GIMBAL_YAW_MOVE_SPEED_SET_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(0.0),
    end: Some(540.0),
    decimal: 0,
    scale: 1.0,
    delta: 0.0,
    unit: "°/s",
};

pub const GIMBAL_ATTI_PITCH_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 2,
    scale: 1.0,
    delta: 0.0,
    unit: "°",
};

pub const GIMBAL_ATTI_YAW_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: None,
    end: None,
    decimal: 2,
    scale: 10.0,
    delta: 0.0,
    unit: "°",
};

pub const GRIPPER_POWER_CONVERTOR: UnitConvertor<f32> = UnitConvertor {
    start: Some(1.0),
    end: Some(100.0),
    decimal: 0,
    scale: 6.6,
    delta: 0.0,
    unit: "",
};

// TODO: default value
pub struct UnitConvertor<V> {
    start: Option<V>,
    end: Option<V>,
    decimal: i32,
    scale: V,
    delta: V,
    unit: &'static str,
}

impl<V> UnitConvertor<V>
where
    V: PartialOrd<V> + MulAssign<V> + DivAssign<V> + AddAssign<V> + SubAssign<V> + MaybeRound,
{
    pub fn check(&self, mut v: V) -> V {
        if let Some(lower) = self.start.as_ref() {
            if let Some(Ordering::Less) = v.partial_cmp(lower) {
                v = *lower;
            }
        }

        if let Some(upper) = self.end.as_ref() {
            if let Some(Ordering::Greater) = v.partial_cmp(upper) {
                v = *upper;
            }
        }

        v
    }

    pub fn val2proto<PV: MaybeFrom<V>>(&self, mut v: V) -> Result<PV> {
        v = self.check(v);
        v *= self.scale;
        v += self.delta;
        v.round(self.decimal)
            .and_then(MaybeFrom::maybe_from)
            .map_err(|ie| Error::InvalidValue {
                unit: self.unit,
                inner: Box::new(ie),
            })
    }

    pub fn proto2val<PV: MaybeInto<V>>(&self, pv: PV) -> Result<V> {
        let mut v = pv.maybe_into().map_err(|ie| Error::InvalidValue {
            unit: self.unit,
            inner: Box::new(ie),
        })?;

        v -= self.delta;
        v /= self.scale;

        v.round(self.decimal).map_err(|ie| Error::InvalidValue {
            unit: self.unit,
            inner: Box::new(ie),
        })?;
        Ok(self.check(v))
    }
}
