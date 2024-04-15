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

use crate::core::{
    ControllerIdReport, ControllerType, ExtHdReport, ExtReport, EXT_I2C_ADDR,
    INTERMESSAGE_DELAY_MICROSEC_U32,
};
use embedded_hal_async;

#[cfg(feature = "defmt_print")]
use defmt;

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub enum AsyncImplError {
    I2C,
    InvalidInputData,
    Error,
    ParseError,
}

pub struct InterfaceAsync<I2C, Delay> {
    i2cdev: I2C,
    delay: Delay,
}

// use crate::nunchuk;
impl<I2C, Delay> InterfaceAsync<I2C, Delay>
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
        Self { i2cdev, delay }
    }

    pub(super) async fn delay_us(&mut self, micros: u32) {
        self.delay.delay_us(micros).await
    }

    /// Read the button/axis data from the classic controller
    pub(super) async fn read_ext_report(&mut self) -> Result<ExtReport, AsyncImplError> {
        self.start_sample().await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .await
            .map_err(|_| AsyncImplError::I2C)
            .and(Ok(buffer))
    }

    /// Read a high-resolution version of the button/axis data from the classic controller
    pub(super) async fn read_hd_report(&mut self) -> Result<ExtHdReport, AsyncImplError> {
        self.start_sample().await?;
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        let mut buffer: ExtHdReport = ExtHdReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .await
            .map_err(|_| AsyncImplError::I2C)
            .and(Ok(buffer))
    }

    /// Send the init sequence to the Wii extension controller
    ///
    /// This could be a bit faster with DelayUs, but since you only init once we'll re-use delay_ms
    pub(super) async fn init(&mut self) -> Result<(), AsyncImplError> {
        // Extension controllers by default will use encrypted communication, as that is what the Wii does.
        // We can disable this encryption by writing some magic values
        // This is described at https://wiibrew.org/wiki/Wiimote/Extension_Controllers#The_New_Way

        // Reset to base register first - this should recover a controller in a weird state.
        // Use longer delays here than normal reads - the system seems more unreliable performing these commands
        self.delay_us(100_000).await;
        self.set_read_register_address_with_delay(0).await?;
        self.set_register_with_delay(0xF0, 0x55).await?;
        self.set_register_with_delay(0xFB, 0x00).await?;
        self.delay_us(100_000).await;
        Ok(())
    }

    /// Switch the driver from standard to hi-resolution reporting
    ///
    /// This enables the controllers high-resolution report data mode, which returns each
    /// analogue axis as a u8, rather than packing smaller integers in a structure.
    /// If your controllers supports this mode, you should use it. It is much better.
    pub(super) async fn enable_hires(&mut self) -> Result<(), AsyncImplError> {
        self.set_register_with_delay(0xFE, 0x03).await?;
        self.delay_us(100_000).await;
        Ok(())
    }

    /// Set the cursor position for the next i2c read
    ///
    /// This hardware has a range of 100 registers and automatically
    /// increments the register read postion on each read operation, and also on
    /// every write operation.
    /// This should be called before a read operation to ensure you get the correct data
    pub(super) async fn set_read_register_address(
        &mut self,
        byte0: u8,
    ) -> Result<(), AsyncImplError> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[byte0])
            .await
            .map_err(|_| AsyncImplError::I2C)
            .and(Ok(()))
    }

    pub(super) async fn set_read_register_address_with_delay(
        &mut self,
        byte0: u8,
    ) -> Result<(), AsyncImplError> {
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        let res = self.set_read_register_address(byte0);
        res.await
    }

    /// Set a single register at target address
    pub(super) async fn set_register(&mut self, addr: u8, byte1: u8) -> Result<(), AsyncImplError> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[addr, byte1])
            .await
            .map_err(|_| AsyncImplError::I2C)
            .and(Ok(()))
    }

    pub(super) async fn set_register_with_delay(
        &mut self,
        addr: u8,
        byte1: u8,
    ) -> Result<(), AsyncImplError> {
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        let res = self.set_register(addr, byte1);
        res.await
    }

    pub(super) async fn read_id(&mut self) -> Result<ControllerIdReport, AsyncImplError> {
        self.set_read_register_address(0xfa).await?;
        let i2c_id = self.read_ext_report().await?;
        Ok(i2c_id)
    }

    pub(super) async fn identify_controller(
        &mut self,
    ) -> Result<Option<ControllerType>, AsyncImplError> {
        let i2c_id = self.read_id().await?;
        Ok(crate::core::identify_controller(i2c_id))
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    pub(super) async fn start_sample(&mut self) -> Result<(), AsyncImplError> {
        self.set_read_register_address(0x00).await?;
        Ok(())
    }
}
