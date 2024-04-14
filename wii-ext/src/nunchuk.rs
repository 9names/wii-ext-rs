// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// All the bugs are Copyright 2021, 9names.

// TODO: nunchuk technically supports HD report, but the last two bytes will be zeroes
// work out if it's worth supporting that

use crate::core::nunchuk::{CalibrationData, NunchukReading, NunchukReadingCalibrated};
use crate::ControllerIdReport;
use crate::ControllerType;
use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC_U32 as INTERMESSAGE_DELAY_MICROSEC;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::{I2c, SevenBitAddress};

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

pub struct Nunchuk<I2C> {
    i2cdev: I2C,
    calibration: CalibrationData,
}

impl<T, E> Nunchuk<T>
where
    T: I2c<SevenBitAddress, Error = E>,
{
    /// Create a new Wii Nunchuk
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new<D: DelayNs>(i2cdev: T, delay: &mut D) -> Result<Nunchuk<T>, Error<E>> {
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
    pub fn update_calibration<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        let data = self.read_report_blocking(delay)?;

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
    pub fn init<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        // These registers must be written to disable encryption.; the documentation is a bit
        // lacking but it appears this is some kind of handshake to
        // perform unencrypted data tranfers
        // Double all the delays here, we sometimes get connection issues otherwise
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xF0, 0x55)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFB, 0x00)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.update_calibration(delay)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        Ok(())
    }

    fn read_id(&mut self) -> Result<ControllerIdReport, Error<E>> {
        self.set_read_register_address(0xfa)?;
        let i2c_id = self.read_report()?;
        Ok(i2c_id)
    }

    pub fn identify_controller(&mut self) -> Result<Option<ControllerType>, Error<E>> {
        let i2c_id = self.read_id()?;
        Ok(crate::common::identify_controller(i2c_id))
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

    /// Simple blocking read helper that will start a sample, wait `INTERMESSAGE_DELAY_MICROSEC`, then read the value
    pub fn read_report_blocking<D: DelayNs>(
        &mut self,
        delay: &mut D,
    ) -> Result<NunchukReading, Error<E>> {
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.start_sample()?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.read_nunchuk()
    }

    /// Do a read, and report axis values relative to calibration
    pub fn read_blocking<D: DelayNs>(
        &mut self,
        delay: &mut D,
    ) -> Result<NunchukReadingCalibrated, Error<E>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_report_blocking(delay)?,
            &self.calibration,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data;
    use embedded_hal_mock::eh1::i2c::{self, Transaction};
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
        let mut mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_idle_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert_eq!(report.joystick_x, 0);
        assert_eq!(report.joystick_y, 0);
        mock.done();
    }

    #[test]
    fn nunchuck_left_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_L.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert!(report.joystick_x < -AXIS_MAX, "x = {}", report.joystick_x);
        assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
        assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
        mock.done();
    }

    #[test]
    fn nunchuck_right_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_R.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert!(report.joystick_x > AXIS_MAX, "x = {}", report.joystick_x);
        assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
        assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
        mock.done();
    }

    #[test]
    fn nunchuck_up_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_U.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert!(report.joystick_y > AXIS_MAX, "y = {}", report.joystick_y);
        assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
        assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
        mock.done();
    }

    #[test]
    fn nunchuck_down_calibrated() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_D.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let idle = NunchukReading::from_data(&test_data::NUNCHUCK_IDLE).unwrap();
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData {
                joystick_x: idle.joystick_x,
                joystick_y: idle.joystick_y,
            },
        };
        let report = nc.read_calibrated_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert!(report.joystick_y < -AXIS_MAX, "y = {}", report.joystick_y);
        assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
        assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
        mock.done();
    }

    #[test]
    fn nunchuck_idle_repeat() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        let report = nc.read_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_btn_c() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_C.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_btn_z() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_Z.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk {
            i2cdev: mock.clone(),
            calibration: CalibrationData::default(),
        };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.button_c);
        assert!(report.button_z);
        mock.done();
    }
}
