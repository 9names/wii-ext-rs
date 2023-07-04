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

use crate::classic_core::*;
use crate::ControllerIdReport;
use crate::ControllerType;
use crate::ExtHdReport;
use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC_U32;
use embedded_hal_async as hal;
use embedded_hal_async::i2c::*;
// use embedded_hal_async::delay::DelayUs;
use embassy_time::{Duration, Timer};

// use core::future::Future;

#[cfg(feature = "defmt_print")]
use defmt;

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub enum ClassicAsyncError {
    I2C,
    InvalidInputData,
    Error,
    ParseError,
}

pub struct ClassicAsync<I2C> {
    i2cdev: I2C,
    hires: bool,
    calibration: CalibrationData,
}

// use crate::nunchuk;
impl<I2C> ClassicAsync<I2C>
where
    I2C: hal::i2c::I2c,
{
    pub type Error = ClassicAsyncError;
    /// Create a new Wii Nunchuck
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new(i2cdev: I2C) -> Self {
        Self {
            i2cdev,
            hires: false,
            calibration: CalibrationData::default(),
        }
    }

    async fn delay_us(&mut self, micros: u32) {
        Timer::after(Duration::from_micros(micros as _)).await
    }

    /// Read the button/axis data from the classic controller
    async fn read_report(&mut self) -> Result<ExtReport, Self::Error> {
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .await
            .map_err(|_| Self::Error::I2C)
            .and(Ok(buffer))
    }

    /// Read a high-resolution version of the button/axis data from the classic controller
    async fn read_hd_report(&mut self) -> Result<ExtHdReport, Self::Error> {
        let mut buffer: ExtHdReport = ExtHdReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .await
            .map_err(|_| Self::Error::I2C)
            .and(Ok(buffer))
    }

    // / Update the stored calibration for this controller
    // /
    // / Since each device will have different tolerances, we take a snapshot of some analog data
    // / to use as the "baseline" center.
    pub async fn update_calibration(&mut self) -> Result<(), Self::Error> {
        let data = self.read_report().await?;
        let data = ClassicReading::from_data(&data).ok_or(Self::Error::InvalidInputData)?;
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
    pub async fn init(&mut self) -> Result<(), Self::Error> {
        // Extension controllers by default will use encrypted communication, as that is what the Wii does.
        // We can disable this encryption by writing some magic values
        // This is described at https://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way

        // Reset to base register first - this should recover a controller in a weird state.
        // Use longer delays here than normal reads - the system seems more unreliable performing these commands
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.set_read_register_address(0).await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.set_register(0xF0, 0x55).await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.set_register(0xFB, 0x00).await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.update_calibration().await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        Ok(())
    }

    /// Switch the driver from standard to hi-resolution reporting
    ///
    /// This enables the controllers high-resolution report data mode, which returns each
    /// analogue axis as a u8, rather than packing smaller integers in a structure.
    /// If your controllers supports this mode, you should use it. It is much better.
    async fn enable_hires(&mut self) -> Result<(), Self::Error> {
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.set_register(0xFE, 0x03).await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        self.hires = true;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32 * 2).await;
        Ok(())
    }

    /// Set the cursor position for the next i2c read
    ///
    /// This hardware has a range of 100 registers and automatically
    /// increments the register read postion on each read operation, and also on
    /// every write operation.
    /// This should be called before a read operation to ensure you get the correct data
    async fn set_read_register_address(&mut self, byte0: u8) -> Result<(), Self::Error> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[byte0])
            .await
            .map_err(|_| Self::Error::I2C)
            .and(Ok(()))
    }

    /// Set a single register at target address
    async fn set_register(&mut self, addr: u8, byte1: u8) -> Result<(), Self::Error> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[addr, byte1])
            .await
            .map_err(|_| Self::Error::I2C)
            .and(Ok(()))
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    async fn start_sample(&mut self) -> Result<(), Self::Error> {
        self.set_read_register_address(0x00).await?;
        Ok(())
    }

    /// poll the controller for the latest data
    async fn read_classic_report(&mut self) -> Result<ClassicReading, Self::Error> {
        if self.hires {
            let buf = self.read_hd_report().await?;
            ClassicReading::from_data(&buf).ok_or(Self::Error::InvalidInputData)
        } else {
            let buf = self.read_report().await?;
            ClassicReading::from_data(&buf).ok_or(Self::Error::InvalidInputData)
        }
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    pub async fn read_report_blocking(&mut self) -> Result<ClassicReading, Self::Error> {
        self.start_sample().await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        self.read_classic_report().await
    }

    /// Do a read, and report axis values relative to calibration
    pub async fn read_blocking(&mut self) -> Result<ClassicReadingCalibrated, Self::Error> {
        Ok(ClassicReadingCalibrated::new(
            self.read_report_blocking().await?,
            &self.calibration,
        ))
    }
}
