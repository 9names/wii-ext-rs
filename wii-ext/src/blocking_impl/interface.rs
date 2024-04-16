use crate::core::{
    ControllerIdReport, ControllerType, ExtHdReport, ExtReport, EXT_I2C_ADDR,
    INTERMESSAGE_DELAY_MICROSEC_U32 as INTERMESSAGE_DELAY_MICROSEC,
};
use embedded_hal::i2c::{I2c, SevenBitAddress};

pub struct Interface<I2C, Delay> {
    i2cdev: I2C,
    delay: Delay,
}

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
/// Errors in this crate
#[derive(Debug)]
pub enum BlockingImplError<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

impl<I2C, E, Delay> Interface<I2C, Delay>
where
    I2C: I2c<SevenBitAddress, Error = E>,
    Delay: embedded_hal::delay::DelayNs,
{
    pub fn new(i2cdev: I2C, delay: Delay) -> Interface<I2C, Delay> {
        Interface { i2cdev, delay }
    }

    /// Recover data members
    pub fn destroy(self) -> (I2C, Delay) {
        (self.i2cdev, self.delay)
    }

    /// Send the init sequence to the Wii extension controller
    pub(super) fn init(&mut self) -> Result<(), BlockingImplError<E>> {
        // Extension controllers by default will use encrypted communication, as that is what the Wii does.
        // We can disable this encryption by writing some magic values
        // This is described at https://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way

        // Reset to base register first - this should recover a controller in a weird state.
        // Use longer delays here than normal reads - the system seems more unreliable performing these commands
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_read_register_address(0)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xF0, 0x55)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFB, 0x00)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        Ok(())
    }

    pub(super) fn read_id(&mut self) -> Result<ControllerIdReport, BlockingImplError<E>> {
        self.set_read_register_address(0xfa)?;
        let i2c_id = self.read_report()?;
        Ok(i2c_id)
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub(super) fn identify_controller(
        &mut self,
    ) -> Result<Option<ControllerType>, BlockingImplError<E>> {
        let i2c_id = self.read_id()?;
        Ok(crate::core::identify_controller(i2c_id))
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    pub(super) fn start_sample(&mut self) -> Result<(), BlockingImplError<E>> {
        self.set_read_register_address(0x00)?;
        Ok(())
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    pub(super) fn start_sample_and_wait(&mut self) -> Result<(), BlockingImplError<E>> {
        self.set_read_register_address(0x00)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        Ok(())
    }

    /// Set the cursor position for the next i2c read
    ///
    /// This hardware has a range of 100 registers and automatically
    /// increments the register read postion on each read operation, and also on
    /// every write operation.
    /// This should be called before a read operation to ensure you get the correct data
    pub(super) fn set_read_register_address(
        &mut self,
        byte0: u8,
    ) -> Result<(), BlockingImplError<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[byte0])
            .map_err(BlockingImplError::I2C)
            .and(Ok(()))
    }

    /// Set a single register at target address
    pub(super) fn set_register(&mut self, addr: u8, byte1: u8) -> Result<(), BlockingImplError<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[addr, byte1])
            .map_err(BlockingImplError::I2C)
            .and(Ok(()))
    }

    /// Read the button/axis data from the classic controller
    pub(super) fn read_report(&mut self) -> Result<ExtReport, BlockingImplError<E>> {
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(BlockingImplError::I2C)
            .and(Ok(buffer))
    }

    pub(super) fn enable_hires(&mut self) -> Result<(), BlockingImplError<E>> {
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFE, 0x03)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        Ok(())
    }

    pub(super) fn disable_hires(&mut self) -> Result<(), BlockingImplError<E>> {
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        self.set_register(0xFE, 0x01)?;
        self.delay.delay_us(INTERMESSAGE_DELAY_MICROSEC * 2);
        Ok(())
    }

    /// Read a high-resolution version of the button/axis data from the classic controller
    pub(super) fn read_hd_report(&mut self) -> Result<ExtHdReport, BlockingImplError<E>> {
        let mut buffer: ExtHdReport = ExtHdReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(BlockingImplError::I2C)
            .and(Ok(buffer))
    }

}
