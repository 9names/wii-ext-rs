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

use crate::core::classic::*;
use crate::ControllerIdReport;
use crate::ControllerType;
use crate::ExtHdReport;
use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::i2c as i2ctrait;

#[cfg(feature = "defmt_print")]
use defmt;

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub enum ClassicError<E> {
    Error(E),
    ParseError,
}

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

pub struct Classic<I2C> {
    i2cdev: I2C,
    hires: bool,
    calibration: CalibrationData,
}

// use crate::nunchuk;
impl<T, E> Classic<T>
where
    T: i2ctrait::Write<Error = E> + i2ctrait::Read<Error = E> + i2ctrait::WriteRead<Error = E>,
{
    /// Create a new Wii Nunchuck
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new<D: DelayUs<u16>>(i2cdev: T, delay: &mut D) -> Result<Classic<T>, Error<E>> {
        let mut classic = Classic {
            i2cdev,
            hires: false,
            calibration: CalibrationData::default(),
        };
        classic.init(delay)?;
        Ok(classic)
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        let data = self.read_report_blocking(delay)?;

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

    /// Set the cursor position for the next i2c read
    ///
    /// This hardware has a range of 100 registers and automatically
    /// increments the register read postion on each read operation, and also on
    /// every write operation.
    /// This should be called before a read operation to ensure you get the correct data
    fn set_read_register_address(&mut self, byte0: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[byte0])
            .map_err(Error::I2C)
            .and(Ok(()))
    }

    /// Set a single register at target address
    fn set_register(&mut self, addr: u8, byte1: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[addr, byte1])
            .map_err(Error::I2C)
            .and(Ok(()))
    }

    /// Read the button/axis data from the classic controller
    fn read_report(&mut self) -> Result<ExtReport, Error<E>> {
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(Error::I2C)
            .and(Ok(buffer))
    }

    /// Read a high-resolution version of the button/axis data from the classic controller
    fn read_hd_report(&mut self) -> Result<ExtHdReport, Error<E>> {
        let mut buffer: ExtHdReport = ExtHdReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(Error::I2C)
            .and(Ok(buffer))
    }

    /// Send the init sequence to the Wii extension controller
    ///
    /// This could be a bit faster with DelayUs, but since you only init once we'll re-use delay_ms
    pub fn init<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        // Extension controllers by default will use encrypted communication, as that is what the Wii does.
        // We can disable this encryption by writing some magic values
        // This is described at https://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way

        // Reset to base register first - this should recover a controller in a weird state.
        // Use longer delays here than normal reads - the system seems more unreliable performing these commands
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_read_register_address(0)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xF0, 0x55)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFB, 0x00)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.update_calibration(delay)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        Ok(())
    }

    /// Switch the driver from standard to hi-resolution reporting
    ///
    /// This enables the controllers high-resolution report data mode, which returns each
    /// analogue axis as a u8, rather than packing smaller integers in a structure.
    /// If your controllers supports this mode, you should use it. It is much better.
    pub fn enable_hires<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFE, 0x03)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.hires = true;
        self.update_calibration(delay)?;
        Ok(())
    }

    /// Switch the driver from hi-resolution to standard reporting reporting
    ///
    /// This disables the controllers high-resolution report data mode
    /// It is assumed that all controllers use 0x01 as the 'standard' mode.
    /// This has only been confirmed for classic and pro-classic controller.
    ///
    /// This function does not work.
    /// TODO: work out why, make it public when it works
    #[allow(dead_code)]
    fn disable_hires<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFE, 0x01)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.hires = false;
        self.update_calibration(delay)?;
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

    /// poll the controller for the latest data
    fn read_classic_report(&mut self) -> Result<ClassicReading, Error<E>> {
        if self.hires {
            let buf = self.read_hd_report()?;
            ClassicReading::from_data(&buf).ok_or(Error::InvalidInputData)
        } else {
            let buf = self.read_report()?;
            ClassicReading::from_data(&buf).ok_or(Error::InvalidInputData)
        }
    }

    /// Simple read helper helper with no delay. Works for testing, not on real hardware
    pub fn read_classic_no_wait(&mut self) -> Result<ClassicReading, Error<E>> {
        self.start_sample()?;
        self.read_classic_report()
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    pub fn read_report_blocking<D: DelayUs<u16>>(
        &mut self,
        delay: &mut D,
    ) -> Result<ClassicReading, Error<E>> {
        self.start_sample()?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.read_classic_report()
    }

    /// Do a read, and report axis values relative to calibration
    pub fn read_blocking<D: DelayUs<u16>>(
        &mut self,
        delay: &mut D,
    ) -> Result<ClassicReadingCalibrated, Error<E>> {
        Ok(ClassicReadingCalibrated::new(
            self.read_report_blocking(delay)?,
            &self.calibration,
        ))
    }
}
