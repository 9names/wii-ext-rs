// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// All the bugs are Copyright 2021, 9names.

// TODO: nunchuk technically supports HD report, but the last two bytes will be zeroes
// work out if it's worth supporting that

use crate::core::nunchuk::{CalibrationData, NunchukReading, NunchukReadingCalibrated};
use crate::interface::Interface;
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

use crate::interface::Error;

pub struct Nunchuk<I2C, DELAY> {
    interface: Interface<I2C, DELAY>,
    calibration: CalibrationData,
}

impl<I2C, ERR, DELAY> Nunchuk<I2C, DELAY>
where
    I2C: I2c<SevenBitAddress, Error = ERR>,
    DELAY: embedded_hal::delay::DelayNs,
{
    /// Create a new Wii Nunchuk
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new(i2cdev: I2C, delay: DELAY) -> Result<Nunchuk<I2C, DELAY>, Error<ERR>> {
        let interface = Interface::new(i2cdev, delay);
        let mut nunchuk = Nunchuk {
            interface,
            calibration: CalibrationData::default(),
        };
        nunchuk.init()?;
        Ok(nunchuk)
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration(&mut self) -> Result<(), Error<ERR>> {
        let data = self.read_report_blocking()?;

        self.calibration = CalibrationData {
            joystick_x: data.joystick_x,
            joystick_y: data.joystick_y,
        };
        Ok(())
    }

    /// Send the init sequence to the Wii extension controller
    pub fn init(&mut self) -> Result<(), Error<ERR>> {
        // These registers must be written to disable encryption.; the documentation is a bit
        // lacking but it appears this is some kind of handshake to
        // perform unencrypted data tranfers
        self.interface.init()?;
        self.update_calibration()
    }

    pub fn identify_controller(&mut self) -> Result<Option<ControllerType>, Error<ERR>> {
        let i2c_id = self.interface.read_id()?;
        Ok(crate::common::identify_controller(i2c_id))
    }

    /// Read the button/axis data from the nunchuk
    fn read_nunchuk(&mut self) -> Result<NunchukReading, Error<ERR>> {
        let buf = self.interface.read_report()?;
        NunchukReading::from_data(&buf).ok_or(Error::InvalidInputData)
    }

    /// Simple blocking read helper that will start a sample, wait `INTERMESSAGE_DELAY_MICROSEC`, then read the value
    pub fn read_report_blocking(&mut self) -> Result<NunchukReading, Error<ERR>> {
        self.interface.start_sample()?;
        self.read_nunchuk()
    }

    /// Do a read, and report axis values relative to calibration
    pub fn read_blocking(&mut self) -> Result<NunchukReadingCalibrated, Error<ERR>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_report_blocking()?,
            &self.calibration,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{self, Transaction},
    };
    /// There's a certain amount of slop around the center position.
    /// Allow up to this range without it being an error
    const ZERO_SLOP: i8 = 5;
    /// The max value at full deflection is ~100, but allow a bit less than that
    const AXIS_MAX: i8 = 90;

    // TODO: work out how to test analogue values from joystick and gyro

    #[test]
    fn nunchuck_idle() {
        let expectations = vec![
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];

        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_idle_calibrated() {
        let expectations = vec![
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Calibration read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        assert_eq!(report.joystick_x, 0);
        assert_eq!(report.joystick_y, 0);
        mock.done();
    }

    #[test]
    fn nunchuck_left_calibrated() {
        let expectations = vec![
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Calibration read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_L.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
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
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Calibration read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_R.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
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
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Calibration read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_U.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
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
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Calibration read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_D.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
        let report = nc.read_blocking().unwrap();
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
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

        let report = nc.read_report_blocking().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        let report = nc.read_report_blocking().unwrap();
        assert!(!report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_btn_c() {
        let expectations = vec![
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_C.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

        let report = nc.read_report_blocking().unwrap();
        assert!(report.button_c);
        assert!(!report.button_z);
        mock.done();
    }

    #[test]
    fn nunchuck_btn_z() {
        let expectations = vec![
            // Reset controller
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            // Init
            Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
            Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
            // Read
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_Z.to_vec()),
        ];
        let mut mock = i2c::Mock::new(&expectations);
        let delay = NoopDelay::new();
        let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

        let report = nc.read_report_blocking().unwrap();
        assert!(!report.button_c);
        assert!(report.button_z);
        mock.done();
    }
}
