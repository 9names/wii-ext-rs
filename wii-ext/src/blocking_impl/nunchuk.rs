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
    pub fn new(i2cdev: I2C, delay: DELAY) -> Result<Nunchuk<I2C, DELAY>, BlockingImplError<ERR>> {
        let interface = Interface::new(i2cdev, delay);
        let mut nunchuk = Nunchuk {
            interface,
            calibration: CalibrationData::default(),
        };
        nunchuk.init()?;
        Ok(nunchuk)
    }

    /// Destroy this driver, recovering the i2c bus and delay used to create it
    pub fn destroy(self) -> (I2C, DELAY) {
        self.interface.destroy()
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration(&mut self) -> Result<(), BlockingImplError<ERR>> {
        let data = self.read_uncalibrated()?;

        self.calibration = CalibrationData {
            joystick_x: data.joystick_x,
            joystick_y: data.joystick_y,
        };
        Ok(())
    }

    /// Send the init sequence to the Nunchuk
    pub fn init(&mut self) -> Result<(), BlockingImplError<ERR>> {
        self.interface.init()?;
        self.update_calibration()
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub fn identify_controller(
        &mut self,
    ) -> Result<Option<ControllerType>, BlockingImplError<ERR>> {
        self.interface.identify_controller()
    }

    /// Do a read, and return button and axis values without applying calibration
    pub fn read_uncalibrated(&mut self) -> Result<NunchukReading, BlockingImplError<ERR>> {
        self.interface.start_sample()?;
        let buf = self.interface.read_report()?;
        NunchukReading::from_data(&buf).ok_or(BlockingImplError::InvalidInputData)
    }

    /// Do a read, and return button and axis values relative to calibration
    pub fn read(&mut self) -> Result<NunchukReadingCalibrated, BlockingImplError<ERR>> {
        Ok(NunchukReadingCalibrated::new(
            self.read_uncalibrated()?,
            &self.calibration,
        ))
    }

}
