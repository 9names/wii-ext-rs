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

use crate::ExtHdReport;
use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::i2c as i2ctrait;

#[cfg(feature = "defmt_print")]
use defmt;

#[derive(Debug)]
pub enum ClassicError<E> {
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

/// Data from a classic controller after it has been deserialized
///
/// In low-res mode, axes with less than 8 bits of range will be
/// scaled to approximate an 8 bit range.
/// in hi-res mode, all axes arleady have 8 bits of range
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct ClassicReading {
    pub joystick_left_x: u8,
    pub joystick_left_y: u8,
    pub joystick_right_x: u8,
    pub joystick_right_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_b: bool,
    pub button_a: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_trigger_l: bool,
    pub button_trigger_r: bool,
    pub button_zl: bool,
    pub button_zr: bool,
    pub button_minus: bool,
    pub button_plus: bool,
    pub button_home: bool,
}

/// Data from a classic controller after calibration data has been applied
///
/// Calibration is done by subtracting the resting values from the current
/// values, which means that going lower on the axis will go negative.
/// Due to this, we now store analog values as signed integers
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct ClassicReadingCalibrated {
    pub joystick_left_x: i8,
    pub joystick_left_y: i8,
    pub joystick_right_x: i8,
    pub joystick_right_y: i8,
    pub trigger_left: i8,
    pub trigger_right: i8,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_b: bool,
    pub button_a: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_trigger_l: bool,
    pub button_trigger_r: bool,
    pub button_zl: bool,
    pub button_zr: bool,
    pub button_minus: bool,
    pub button_plus: bool,
    pub button_home: bool,
}

impl ClassicReadingCalibrated {
    pub fn new(r: ClassicReading, c: &CalibrationData) -> ClassicReadingCalibrated {
        /// Just in case `data` minus `calibration data` is out of range, perform all operations
        /// on i16 and clamp to i8 limits before returning
        fn ext_u8_sub(a: u8, b: u8) -> i8 {
            let res = (a as i16) - (b as i16);
            res.clamp(i8::MIN as i16, i8::MAX as i16) as i8
        }

        ClassicReadingCalibrated {
            joystick_left_x: ext_u8_sub(r.joystick_left_x, c.joystick_left_x),
            joystick_left_y: ext_u8_sub(r.joystick_left_y, c.joystick_left_y),
            joystick_right_x: ext_u8_sub(r.joystick_right_x, c.joystick_right_x),
            joystick_right_y: ext_u8_sub(r.joystick_right_y, c.joystick_right_y),
            trigger_left: ext_u8_sub(r.trigger_left, c.trigger_left),
            trigger_right: ext_u8_sub(r.trigger_right, c.trigger_right),
            dpad_up: r.dpad_up,
            dpad_down: r.dpad_down,
            dpad_left: r.dpad_left,
            dpad_right: r.dpad_right,
            button_b: r.button_b,
            button_a: r.button_a,
            button_x: r.button_x,
            button_y: r.button_y,
            button_trigger_l: r.button_trigger_l,
            button_trigger_r: r.button_trigger_r,
            button_zl: r.button_zl,
            button_zr: r.button_zr,
            button_minus: r.button_minus,
            button_plus: r.button_plus,
            button_home: r.button_home,
        }
    }
}

/// Convert raw data as returned from controller via i2c into buttons and axis fields
#[rustfmt::skip]
fn decode_classic_report(data: &[u8]) -> ClassicReading {
    // Classic mode:
    //  Bit	7	6	5	4	3	2	1	0
    // 	Byte
    // 	0	RX<4:3>	LX<5:0>
    // 	1	RX<2:1>	LY<5:0>
    // 	2	RX<0>	LT<4:3>	RY<4:0>
    // 	3	LT<2:0>	RT<4:0>
    // 	4	BDR	BDD	BLT	B-	BH	B+	BRT	1
    // 	5	BZL	BB	BY	BA	BX	BZR	BDL	BDU
    ClassicReading {
        joystick_left_x:   ClassicReading::scale_6bit_8bit(data[0] & 0b0011_1111),
        joystick_left_y:   ClassicReading::scale_6bit_8bit(data[1] & 0b0011_1111),
        joystick_right_x:  ClassicReading::scale_5bit_8bit(
            ((data[2] & 0b1000_0000) >> 7) |
            ((data[1] & 0b1100_0000) >> 5) |
            ((data[0] & 0b1100_0000) >> 3)
        ),
        joystick_right_y:  ClassicReading::scale_5bit_8bit(data[2] & 0b0001_1111),
        trigger_left:     ClassicReading::scale_5bit_8bit(
            ((data[2] & 0b0110_0000) >> 2) |
            ((data[3] & 0b1110_0000) >> 5)
        ),
        trigger_right:    ClassicReading::scale_5bit_8bit(data[3] & 0b0001_1111),
        dpad_right:       data[4] & 0b1000_0000 == 0,
        dpad_down:        data[4] & 0b0100_0000 == 0,
        button_trigger_l: data[4] & 0b0010_0000 == 0,
        button_minus:     data[4] & 0b0001_0000 == 0,
        button_home:      data[4] & 0b0000_1000 == 0,
        button_plus:      data[4] & 0b0000_0100 == 0,
        button_trigger_r: data[4] & 0b0000_0010 == 0,
        button_zl:        data[5] & 0b1000_0000 == 0,
        button_b:         data[5] & 0b0100_0000 == 0,
        button_y:         data[5] & 0b0010_0000 == 0,
        button_a:         data[5] & 0b0001_0000 == 0,
        button_x:         data[5] & 0b0000_1000 == 0,
        button_zr:        data[5] & 0b0000_0100 == 0,
        dpad_left:        data[5] & 0b0000_0010 == 0,
        dpad_up:          data[5] & 0b0000_0001 == 0,
    }
}

/// Convert high-resolution raw data as returned from controller via i2c into buttons and axis fields
#[rustfmt::skip]
fn decode_classic_hd_report(data: &[u8]) -> ClassicReading {
    // High precision mode:
    // Bit    7    6    5    4    3    2    1    0
    // Byte
    // 0      LX<7:0>
    // 1      RX<7:0>
    // 2      LY<7:0>
    // 3      RY<7:0>
    // 4      LT<7:0>
    // 5      RT<7:0>
    // 6      BDR  BDD  BLT  B-   BH   B+   BRT  1
    // 7      BZL  BB   BY   BA   BX   BZR  BDL  BDU
    ClassicReading {
        joystick_left_x:   data[0],
        joystick_right_x:  data[1],
        joystick_left_y:   data[2],
        joystick_right_y:  data[3],
        trigger_left:     data[4],
        trigger_right:    data[5],
        dpad_right:       data[6] & 0b1000_0000 == 0,
        dpad_down:        data[6] & 0b0100_0000 == 0,
        button_trigger_l: data[6] & 0b0010_0000 == 0,
        button_minus:     data[6] & 0b0001_0000 == 0,
        button_home:      data[6] & 0b0000_1000 == 0,
        button_plus:      data[6] & 0b0000_0100 == 0,
        button_trigger_r: data[6] & 0b0000_0010 == 0,
        button_zl:        data[7] & 0b1000_0000 == 0,
        button_b:         data[7] & 0b0100_0000 == 0,
        button_y:         data[7] & 0b0010_0000 == 0,
        button_a:         data[7] & 0b0001_0000 == 0,
        button_x:         data[7] & 0b0000_1000 == 0,
        button_zr:        data[7] & 0b0000_0100 == 0,
        dpad_left:        data[7] & 0b0000_0010 == 0,
        dpad_up:          data[7] & 0b0000_0001 == 0,
    }
}

/// Relaxed/Center positions for each axis
///
/// These are used to calculate the relative deflection of each access from their center point
#[derive(Default)]
pub struct CalibrationData {
    pub joystick_left_x: u8,
    pub joystick_left_y: u8,
    pub joystick_right_x: u8,
    pub joystick_right_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
}

impl ClassicReading {
    #[cfg(test)]
    /// Helper function for testing digital pin status
    /// This should work for all different classic controllers
    /// Testing analogue is harder, will have to think about testing those.
    pub fn assert_digital_eq(&self, other: ClassicReading) {
        assert_eq!(self.button_a, other.button_a);
        assert_eq!(self.button_b, other.button_b);
        assert_eq!(self.button_x, other.button_x);
        assert_eq!(self.button_y, other.button_y);
        assert_eq!(self.button_trigger_l, other.button_trigger_l);
        assert_eq!(self.button_trigger_r, other.button_trigger_r);
        assert_eq!(self.button_zl, other.button_zl);
        assert_eq!(self.button_zr, other.button_zr);
        assert_eq!(self.button_home, other.button_home);
        assert_eq!(self.button_plus, other.button_plus);
        assert_eq!(self.button_minus, other.button_minus);
    }

    /// Some axis' data is u5, scale it to u8 for convenience
    fn scale_5bit_8bit(reading: u8) -> u8 {
        // TODO: better math here, move this somewhere common
        ((reading as u32 * u8::MAX as u32) / 31) as u8
    }

    /// Some axis' data is u6, scale it to u8 for convenience
    fn scale_6bit_8bit(reading: u8) -> u8 {
        // TODO: better math here, move this somewhere common
        ((reading as u32 * u8::MAX as u32) / 63) as u8
    }

    /// Convert from a wii-ext report into controller data
    pub fn from_data(data: &[u8]) -> Option<ClassicReading> {
        if data.len() == 6 {
            // Classic mode:
            Some(decode_classic_report(data))
        } else if data.len() == 8 {
            // High precision mode:
            Some(decode_classic_hd_report(data))
        } else {
            None
        }
    }
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

    fn read_id(&mut self) -> Result<ExtReport, Error<E>> {
        self.set_read_register_address(0xfa)?;
        let i2c_id = self.read_report()?;
        Ok(i2c_id)
    }

    //TODO: make a real enum for this
    // for now:
    // 0 unhandled controller
    // 1 not ext
    // 2 nunchuck
    // 3 wii classic
    // 4 wii classic pro
    // IDs:
    // pub const NUNCHUCK_ID: ExtReport = [0, 0, 164, 32, 0, 0];
    // pub const CLASSIC_ID: ExtReport = [0, 0, 164, 32, 3, 1];
    // pub const PRO_ID:     ExtReport = [1, 0, 164, 32, 1, 1];
    pub fn identify_controller(&mut self) -> Result<u8, Error<E>> {
        let i2c_id = self.read_id()?;

        if i2c_id[2] != 0xA4 || i2c_id[3] != 0x20 {
            // Not an extension controller
            Ok(1)
        } else if i2c_id[0] == 0 && i2c_id[1] == 0 && i2c_id[4] == 0 && i2c_id[5] == 0 {
            // It's a nunchuck
            Ok(2)
        } else if i2c_id[0] == 0 && i2c_id[1] == 0 && i2c_id[4] == 3 && i2c_id[5] == 1 {
            // It's a wii classic controller
            Ok(3)
        } else if i2c_id[0] == 1 && i2c_id[1] == 0 && i2c_id[4] == 1 && i2c_id[5] == 1 {
            // It's a wii classic pro (or compatible) controller
            // This is most wii classic extension controllers (NES/SNES/Clones)
            Ok(4)
        } else {
            Ok(0)
        }
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
