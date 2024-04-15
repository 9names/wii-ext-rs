// See https://www.raphnet-tech.com/support/classic_controller_high_res/ for data on high-precision mode

// Abridged version of the above:
// To enable High Resolution Mode, you simply write 0x03 to address 0xFE in the extension controller memory.
// Then you poll the controller by reading 8 bytes at address 0x00 instead of only 6.
// You can also restore the original format by writing the original value back to address 0xFE at any time.
//
// Classic mode:
// http://wiibrew.org/wiki/Wiimote/Extension_Controllers/Classic_Controller
//
// See `decode_classic_report` and `decode_classic_hd_report` for data format

use crate::async_impl::interface::{AsyncImplError, InterfaceAsync};
use crate::core::classic::*;
use crate::core::{ControllerIdReport, ControllerType};
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
    /// Create a new Wii Nunchuck
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new(i2cdev: I2C, delay: Delay) -> Self {
        let interface = InterfaceAsync::new(i2cdev, delay);
        Self {
            interface,
            hires: false,
            calibration: CalibrationData::default(),
        }
    }

    // / Update the stored calibration for this controller
    // /
    // / Since each device will have different tolerances, we take a snapshot of some analog data
    // / to use as the "baseline" center.
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

    /// Send the init sequence to the Wii extension controller
    ///
    /// This could be a bit faster with DelayUs, but since you only init once we'll re-use delay_ms
    pub async fn init(&mut self) -> Result<(), AsyncImplError> {
        // Extension controllers by default will use encrypted communication, as that is what the Wii does.
        // We can disable this encryption by writing some magic values
        // This is described at https://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way

        // Reset to base register first - this should recover a controller in a weird state.
        // Use longer delays here than normal reads - the system seems more unreliable performing these commands
        self.interface.init().await?;
        self.update_calibration().await?;
        Ok(())
    }

    /// poll the controller for the latest data
    async fn read_classic_report(&mut self) -> Result<ClassicReading, AsyncImplError> {
        if self.hires {
            let buf = self.interface.read_hd_report().await?;
            ClassicReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
        } else {
            let buf = self.interface.read_ext_report().await?;
            ClassicReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
        }
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    async fn read_report(&mut self) -> Result<ClassicReading, AsyncImplError> {
        self.read_classic_report().await
    }

    /// Do a read, and report axis values relative to calibration
    pub async fn read(&mut self) -> Result<ClassicReadingCalibrated, AsyncImplError> {
        Ok(ClassicReadingCalibrated::new(
            self.read_classic_report().await?,
            &self.calibration,
        ))
    }

    pub async fn enable_hires(&mut self) -> Result<(), AsyncImplError> {
        self.interface.enable_hires().await
    }

    pub async fn read_id(&mut self) -> Result<ControllerIdReport, AsyncImplError> {
        self.interface.read_id().await
    }

    pub async fn identify_controller(&mut self) -> Result<Option<ControllerType>, AsyncImplError> {
        self.interface.identify_controller().await
    }
}
