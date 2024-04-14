#[cfg(feature = "defmt_print")]
use defmt;

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub struct NunchukReading {
    pub joystick_x: u8,
    pub joystick_y: u8,
    pub accel_x: u16, // 10-bit
    pub accel_y: u16, // 10-bit
    pub accel_z: u16, // 10-bit
    pub button_c: bool,
    pub button_z: bool,
}

impl NunchukReading {
    pub fn from_data(data: &[u8]) -> Option<NunchukReading> {
        if data.len() < 6 {
            None
        } else {
            Some(NunchukReading {
                joystick_x: data[0],
                joystick_y: data[1],
                accel_x: (u16::from(data[2]) << 2) | ((u16::from(data[5]) >> 6) & 0b11),
                accel_y: (u16::from(data[3]) << 2) | ((u16::from(data[5]) >> 4) & 0b11),
                accel_z: (u16::from(data[4]) << 2) | ((u16::from(data[5]) >> 2) & 0b11),
                button_c: (data[5] & 0b10) == 0,
                button_z: (data[5] & 0b01) == 0,
            })
        }
    }
}

/// Relaxed/Center positions for each axis
///
/// These are used to calculate the relative deflection of each access from their center point
#[derive(Default)]
pub struct CalibrationData {
    pub joystick_x: u8,
    pub joystick_y: u8,
}

/// Data from a Nunchuk after calibration data has been applied
///
/// Calibration is done by subtracting the resting values from the current
/// values, which means that going lower on the axis will go negative.
/// Due to this, we now store analog values as signed integers
///
/// We'll only calibrate the joystick axes, leave accelerometer readings as-is
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct NunchukReadingCalibrated {
    pub joystick_x: i8,
    pub joystick_y: i8,
    pub accel_x: u16, // 10-bit
    pub accel_y: u16, // 10-bit
    pub accel_z: u16, // 10-bit
    pub button_c: bool,
    pub button_z: bool,
}

impl NunchukReadingCalibrated {
    pub fn new(r: NunchukReading, c: &CalibrationData) -> NunchukReadingCalibrated {
        /// Just in case `data` minus `calibration data` is out of range, perform all operations
        /// on i16 and clamp to i8 limits before returning
        fn ext_u8_sub(a: u8, b: u8) -> i8 {
            let res = (a as i16) - (b as i16);
            res.clamp(i8::MIN as i16, i8::MAX as i16) as i8
        }

        NunchukReadingCalibrated {
            joystick_x: ext_u8_sub(r.joystick_x, c.joystick_x),
            joystick_y: ext_u8_sub(r.joystick_y, c.joystick_y),
            accel_x: r.accel_x,
            accel_y: r.accel_y, // 10-bit
            accel_z: r.accel_z, // 10-bit
            button_c: r.button_c,
            button_z: r.button_z,
        }
    }
}
