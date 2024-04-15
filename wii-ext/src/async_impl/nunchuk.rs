// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// All the bugs are Copyright 2024, 9names.

// Nunchuk technically supports HD report, but the last two bytes will be zeroes
// There's no benefit, so we're leaving that unimplemented
use crate::async_impl::interface::{ClassicAsyncError as AsyncImplError, InterfaceAsync};
use crate::core::nunchuk::*;
use crate::core::{ControllerIdReport, ControllerType};
use embedded_hal_async;

pub struct NunchukAsync<I2C, Delay> {
    interface: InterfaceAsync<I2C, Delay>,
    calibration: CalibrationData,
}

impl<I2C, Delay> NunchukAsync<I2C, Delay>
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
            calibration: CalibrationData::default(),
        }
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
    async fn read_classic_report(&mut self) -> Result<NunchukReading, AsyncImplError> {
        let buf = self.interface.read_ext_report().await?;
        NunchukReading::from_data(&buf).ok_or(AsyncImplError::InvalidInputData)
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    async fn read_report(&mut self) -> Result<NunchukReading, AsyncImplError> {
        self.read_classic_report().await
    }

    /// Do a read, and report axis values relative to calibration
    pub async fn read(&mut self) -> Result<NunchukReadingCalibrated, AsyncImplError> {
        Ok(NunchukReadingCalibrated::new(
            self.read_classic_report().await?,
            &self.calibration,
        ))
    }

    pub async fn read_id(&mut self) -> Result<ControllerIdReport, AsyncImplError> {
        self.interface.read_id().await
    }

    pub async fn identify_controller(&mut self) -> Result<Option<ControllerType>, AsyncImplError> {
        self.interface.identify_controller().await
    }
}
