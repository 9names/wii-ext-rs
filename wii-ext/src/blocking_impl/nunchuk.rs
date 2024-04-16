// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// All the bugs are Copyright 2021, 9names.

// TODO: nunchuk technically supports HD report, but the last two bytes will be zeroes
// work out if it's worth supporting that

use crate::blocking_impl::interface::{BlockingImplError, Interface};
use crate::core::nunchuk::{CalibrationData, NunchukReading, NunchukReadingCalibrated};
use crate::core::ControllerType;
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[derive(Debug)]
pub enum NunchukError<E> {
    Error(E),
    ParseError,
}

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
    pub fn new(i2cdev: I2C, delay: DELAY) -> Result<Nunchuk<I2C, DELAY>, BlockingImplError<ERR>> {
        let interface = Interface::new(i2cdev, delay);
        let mut nunchuk = Nunchuk {
            interface,
            calibration: CalibrationData::default(),
        };
        nunchuk.init()?;
        Ok(nunchuk)
    }

    /// Recover data members
    pub fn destroy(self) -> (I2C, DELAY) {
        self.interface.destroy()
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration(&mut self) -> Result<(), BlockingImplError<ERR>> {
        let data = self.read_report_blocking()?;

        self.calibration = CalibrationData {
            joystick_x: data.joystick_x,
            joystick_y: data.joystick_y,
        };
        Ok(())
    }

    /// Send the init sequence to the Wii extension controller
    pub fn init(&mut self) -> Result<(), BlockingImplError<ERR>> {
        // These registers must be written to disable encryption.; the documentation is a bit
        // lacking but it appears this is some kind of handshake to
        // perform unencrypted data tranfers
        self.interface.init()?;
        self.update_calibration()
    }

    pub fn identify_controller(
        &mut self,
    ) -> Result<Option<ControllerType>, BlockingImplError<ERR>> {
        self.interface.identify_controller()
    }

    /// Read the button/axis data from the nunchuk
    fn read_nunchuk(&mut self) -> Result<NunchukReading, BlockingImplError<ERR>> {
        let buf = self.interface.read_report()?;
        NunchukReading::from_data(&buf).ok_or(BlockingImplError::InvalidInputData)
    }

    /// Simple blocking read helper that will start a sample, wait `INTERMESSAGE_DELAY_MICROSEC`, then read the value
    pub fn read_report_blocking(&mut self) -> Result<NunchukReading, BlockingImplError<ERR>> {
        self.interface.start_sample()?;
        self.read_nunchuk()
    }

    /// Do a read, and report axis values relative to calibration
    pub fn read_blocking(&mut self) -> Result<NunchukReadingCalibrated, BlockingImplError<ERR>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_report_blocking()?,
            &self.calibration,
        ))
    }

}
