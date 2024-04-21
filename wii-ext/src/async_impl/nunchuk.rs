use crate::async_impl::interface::{AsyncImplError, InterfaceAsync};
use crate::core::nunchuk::*;
use crate::core::ControllerType;
use embedded_hal_async;

pub struct Nunchuk<I2C, Delay> {
    interface: InterfaceAsync<I2C, Delay>,
    calibration: CalibrationData,
}

impl<I2C, Delay> Nunchuk<I2C, Delay>
where
    I2C: embedded_hal_async::i2c::I2c,
    Delay: embedded_hal_async::delay::DelayNs,
{
    /// Create a new Wii Nunchuck
    pub fn new(i2cdev: I2C, delay: Delay) -> Self {
        let interface = InterfaceAsync::new(i2cdev, delay);
        Self {
            interface,
            calibration: CalibrationData::default(),
        }
    }

    /// Destroy this driver, recovering the i2c bus and delay used to create it
    pub fn destroy(self) -> (I2C, Delay) {
        self.interface.destroy()
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub async fn update_calibration(&mut self) -> Result<(), AsyncImplError> {
        let data = self.read_report().await?;
        self.calibration = CalibrationData {
            joystick_x: data.joystick_x,
            joystick_y: data.joystick_y,
        };
        Ok(())
    }

    /// Send the init sequence to the controller and calibrate it
    pub async fn init(&mut self) -> Result<(), AsyncImplError> {
        self.interface.init().await?;
        self.update_calibration().await?;
        Ok(())
    }

    /// poll the controller for the latest data
    async fn read_report(&mut self) -> Result<NunchukReading, AsyncImplError> {
        let buf = self.interface.read_ext_report().await?;
        NunchukReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
    }

    /// Do a read, and report axis values relative to calibration
    pub async fn read(&mut self) -> Result<NunchukReadingCalibrated, AsyncImplError> {
        Ok(NunchukReadingCalibrated::new(
            self.read_report().await?,
            &self.calibration,
        ))
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub async fn identify_controller(&mut self) -> Result<Option<ControllerType>, AsyncImplError> {
        self.interface.identify_controller().await
    }
}
