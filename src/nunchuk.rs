// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// All the bugs are Copyright 2021, 9names.

use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC;
// nunchuk technically supports HD report, but the last two bytes will be zeroes
// TODO: work out if it's worth supporting that
// TODO: add support for the
// use crate::ExtHdReport;
use embedded_hal::blocking::delay::DelayUs;

#[cfg(feature = "defmt_print")]
use defmt;

#[derive(Debug)]
pub enum NunchukError<E> {
    Error(E),
    ParseError,
}

/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub struct NunchukReading {
    pub joystick_x: u8,
    pub joystick_y: u8,
    pub accel_x: u16, // 10-bit
    pub accel_y: u16, // 10-bit
    pub accel_z: u16, // 10-bit
    pub c_button_pressed: bool,
    pub z_button_pressed: bool,
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
                c_button_pressed: (data[5] & 0b10) == 0,
                z_button_pressed: (data[5] & 0b01) == 0,
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
    pub c_button_pressed: bool,
    pub z_button_pressed: bool,
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
            c_button_pressed: r.c_button_pressed,
            z_button_pressed: r.z_button_pressed,
        }
    }
}

pub struct Nunchuk<I2C> {
    i2cdev: I2C,
    calibration: CalibrationData,
}

use embedded_hal::blocking::i2c as i2ctrait;
impl<T, E> Nunchuk<T>
where
    T: i2ctrait::Write<Error = E> + i2ctrait::Read<Error = E> + i2ctrait::WriteRead<Error = E>,
{
    /// Create a new Wii Nunchuk
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new<D: DelayUs<u16>>(i2cdev: T, delay: &mut D) -> Result<Nunchuk<T>, Error<E>> {
        let mut nunchuk = Nunchuk {
            i2cdev,
            calibration: CalibrationData::default(),
        };
        nunchuk.init(delay)?;
        Ok(nunchuk)
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        let data = self.read_blocking(delay)?;

        self.calibration = CalibrationData {
            joystick_x: data.joystick_x,
            joystick_y: data.joystick_y,
        };
        Ok(())
    }

    fn set_read_register_address(&mut self, address: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[address])
            .map_err(Error::I2C)
    }

    fn set_register(&mut self, reg: u8, val: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[reg, val])
            .map_err(Error::I2C)
    }

    fn read_report(&mut self) -> Result<ExtReport, Error<E>> {
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(Error::I2C)
            .and(Ok(buffer))
    }

    /// Send the init sequence to the Wii extension controller
    pub fn init<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        // These registers must be written to disable encryption.; the documentation is a bit
        // lacking but it appears this is some kind of handshake to
        // perform unencrypted data tranfers
        self.set_register(0xF0, 0x55)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.set_register(0xFB, 0x00)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.update_calibration(delay)?;
        Ok(())
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    fn start_sample(&mut self) -> Result<(), Error<E>> {
        self.set_read_register_address(0x00)?;

        Ok(())
    }

    /// Read the button/axis data from the nunchuk
    fn read_nunchuk(&mut self) -> Result<NunchukReading, Error<E>> {
        let buf = self.read_report()?;
        NunchukReading::from_data(&buf).ok_or(Error::InvalidInputData)
    }

    /// Simple helper with no delay. Should work for testing, not sure if it will function on hardware
    pub fn read_no_wait(&mut self) -> Result<NunchukReading, Error<E>> {
        self.start_sample()?;
        self.read_nunchuk()
    }

    /// Simple helper with no delay. Should work for testing, not sure if it will function on hardware
    pub fn read_calibrated_no_wait(&mut self) -> Result<NunchukReadingCalibrated, Error<E>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_no_wait()?,
            &self.calibration,
        ))
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    pub fn read_blocking<D: DelayUs<u16>>(
        &mut self,
        delay: &mut D,
    ) -> Result<NunchukReading, Error<E>> {
        self.start_sample()?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.read_nunchuk()
    }

    /// Do a read, and report axis values relative to calibration
    pub fn read_blocking_calibrated<D: DelayUs<u16>>(
        &mut self,
        delay: &mut D,
    ) -> Result<NunchukReadingCalibrated, Error<E>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_blocking(delay)?,
            &self.calibration,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data;
    use embedded_hal_mock::i2c::{self, Transaction};
    /// There's a certain amount of slop around the center position.
    /// Allow up to this range without it being an error
    const ZERO_SLOP: i8 = 5;
    /// The max value at full deflection is ~100, but allow a bit less than that
    const AXIS_MAX: i8 = 90;

    // TODO: work out how to test analogue values from joystick and gyro

    #[test]
    fn nunchuck_idle() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_idle_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        assert_eq!(report.joystick_x, 0);
        assert_eq!(report.joystick_y, 0);
    }

    #[test]
    fn nunchuck_left_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_L.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        assert!(report.joystick_x < -AXIS_MAX, "x = {}", report.joystick_x);
        assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
        assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
    }

    #[test]
    fn nunchuck_right_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_R.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        assert!(report.joystick_x > AXIS_MAX, "x = {}", report.joystick_x);
        assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
        assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
    }

    #[test]
    fn nunchuck_up_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_U.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        assert!(report.joystick_y > AXIS_MAX, "y = {}", report.joystick_y);
        assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
        assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
    }

    #[test]
    fn nunchuck_down_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_D.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        assert!(report.joystick_y < -AXIS_MAX, "y = {}", report.joystick_y);
        assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
        assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
    }

    #[test]
    fn nunchuck_idle_repeat() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_btn_c() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_C.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_btn_z() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_Z.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock,
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(report.z_button_pressed);
    }
}
