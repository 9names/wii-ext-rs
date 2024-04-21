use crate::blocking_impl::interface::{BlockingImplError, Interface};
use crate::core::classic::{CalibrationData, ClassicReading, ClassicReadingCalibrated};
use crate::core::ControllerType;
use embedded_hal::i2c::I2c;

#[cfg(feature = "defmt_print")]
use defmt;
use embedded_hal::i2c::SevenBitAddress;

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub enum ClassicError<E> {
    Error(E),
    ParseError,
}

pub struct Classic<I2C, DELAY> {
    interface: Interface<I2C, DELAY>,
    hires: bool,
    calibration: CalibrationData,
}

impl<T, E, DELAY> Classic<T, DELAY>
where
    T: I2c<SevenBitAddress, Error = E>,
    DELAY: embedded_hal::delay::DelayNs,
{
    /// Create a new Wii Classic Controller
    pub fn new(i2cdev: T, delay: DELAY) -> Result<Classic<T, DELAY>, BlockingImplError<E>> {
        let interface = Interface::new(i2cdev, delay);
        let mut classic = Classic {
            interface,
            hires: false,
            calibration: CalibrationData::default(),
        };
        classic.init()?;
        Ok(classic)
    }

    /// Destroy this driver, recovering the i2c bus and delay used to create it
    pub fn destroy(self) -> (T, DELAY) {
        self.interface.destroy()
    }

    /// Update the stored calibration for this controller
    ///
    /// Since each device will have different tolerances, we take a snapshot of some analog data
    /// to use as the "baseline" center.
    pub fn update_calibration(&mut self) -> Result<(), BlockingImplError<E>> {
        let data = self.read_uncalibrated()?;

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

    /// Send the init sequence to the controller
    pub fn init(&mut self) -> Result<(), BlockingImplError<E>> {
        self.interface.init()?;
        self.update_calibration()?;
        Ok(())
    }

    /// Switch the driver from standard to hi-resolution reporting
    ///
    /// This enables the controllers high-resolution report data mode, which returns each
    /// analogue axis as a u8, rather than packing smaller integers in a structure.
    /// If your controllers supports this mode, you should use it. It is much better.
    pub fn enable_hires(&mut self) -> Result<(), BlockingImplError<E>> {
        self.interface.enable_hires()?;
        self.hires = true;
        self.update_calibration()?;
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
    fn disable_hires(&mut self) -> Result<(), BlockingImplError<E>> {
        self.interface.disable_hires()?;
        self.hires = false;
        self.update_calibration()?;
        Ok(())
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub fn identify_controller(&mut self) -> Result<Option<ControllerType>, BlockingImplError<E>> {
        self.interface.identify_controller()
    }

    /// Do a read, and return button and axis values without applying calibration
    pub fn read_uncalibrated(&mut self) -> Result<ClassicReading, BlockingImplError<E>> {
        self.interface.start_sample_and_wait()?;
        if self.hires {
            let buf = self.interface.read_hd_report()?;
            ClassicReading::from_data(&buf).ok_or(BlockingImplError::InvalidInputData)
        } else {
            let buf = self.interface.read_report()?;
            ClassicReading::from_data(&buf).ok_or(BlockingImplError::InvalidInputData)
        }
    }

    /// Do a read, and return button and axis values relative to calibration
    pub fn read(&mut self) -> Result<ClassicReadingCalibrated, BlockingImplError<E>> {
        Ok(ClassicReadingCalibrated::new(
            self.read_uncalibrated()?,
            &self.calibration,
        ))
    }
}
