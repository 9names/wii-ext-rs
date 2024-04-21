use crate::async_impl::interface::{AsyncImplError, InterfaceAsync};
use crate::core::classic::*;
use crate::core::ControllerType;
use embedded_hal_async;

pub struct ClassicAsync<I2C, Delay> {
    interface: InterfaceAsync<I2C, Delay>,
    hires: bool,
    calibration: CalibrationData,
}

impl<I2C, Delay> ClassicAsync<I2C, Delay>
where
    I2C: embedded_hal_async::i2c::I2c,
    Delay: embedded_hal_async::delay::DelayNs,
{
    /// Create a new Wii Classic Controller
    pub fn new(i2cdev: I2C, delay: Delay) -> Self {
        let interface = InterfaceAsync::new(i2cdev, delay);
        Self {
            interface,
            hires: false,
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
            joystick_left_x: data.joystick_left_x,
            joystick_left_y: data.joystick_left_y,
            joystick_right_x: data.joystick_right_x,
            joystick_right_y: data.joystick_right_y,
            trigger_left: data.trigger_left,
            trigger_right: data.trigger_left,
        };
        Ok(())
    }

    /// Send the init sequence to the controller and calibrate it
    pub async fn init(&mut self) -> Result<(), AsyncImplError> {
        self.interface.init().await?;
        self.update_calibration().await?;
        Ok(())
    }

    /// Read uncalibrated data from the controller
    async fn read_report(&mut self) -> Result<ClassicReading, AsyncImplError> {
        if self.hires {
            let buf = self.interface.read_hd_report().await?;
            ClassicReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
        } else {
            let buf = self.interface.read_ext_report().await?;
            ClassicReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
        }
    }

    /// Do a read, and report axis values relative to calibration
    pub async fn read(&mut self) -> Result<ClassicReadingCalibrated, AsyncImplError> {
        Ok(ClassicReadingCalibrated::new(
            self.read_report().await?,
            &self.calibration,
        ))
    }

    /// Switch the driver from standard to hi-resolution reporting
    ///
    /// This enables the controllers high-resolution report data mode, which returns each
    /// analogue axis as a u8, rather than packing smaller integers in a structure.
    /// If your controllers supports this mode, you should use it. It is much better.
    pub async fn enable_hires(&mut self) -> Result<(), AsyncImplError> {
        self.interface.enable_hires().await
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub async fn identify_controller(&mut self) -> Result<Option<ControllerType>, AsyncImplError> {
        self.interface.identify_controller().await
    }
}
