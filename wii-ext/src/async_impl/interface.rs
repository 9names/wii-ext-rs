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

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct InterfaceAsync<I2C, Delay> {
    i2cdev: I2C,
    delay: Delay,
}

impl<I2C, Delay> InterfaceAsync<I2C, Delay>
where
    I2C: embedded_hal_async::i2c::I2c,
    Delay: embedded_hal_async::delay::DelayNs,
{
    /// Create async interface for wii-extension controller
    pub fn new(i2cdev: I2C, delay: Delay) -> Self {
        Self { i2cdev, delay }
    }

    /// Destroy i2c interface, allowing recovery of i2c and delay
    pub fn destroy(self) -> (I2C, Delay) {
        (self.i2cdev, self.delay)
    }

    /// Access delay stored in interface
    pub(super) async fn delay_us(&mut self, micros: u32) {
        self.delay.delay_us(micros).await
    }

    /// Read report data from the wii-extension controller
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

    /// Read a high-resolution version of the report data from the wii-extension controller
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
    /// This enables the controller's high-resolution report data mode, which returns each
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

    /// Set the cursor position for the next i2c read after a small delay
    ///
    /// This hardware has a range of 100 registers and automatically
    /// increments the register read postion on each read operation, and also on
    /// every write operation.
    /// This should be called before a read operation to ensure you get the correct data
    /// The delay helps ensure that required timings are met
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

    /// Set a single register at target address after a small delay
    pub(super) async fn set_register_with_delay(
        &mut self,
        addr: u8,
        byte1: u8,
    ) -> Result<(), AsyncImplError> {
        self.delay_us(INTERMESSAGE_DELAY_MICROSEC_U32).await;
        let res = self.set_register(addr, byte1);
        res.await
    }

    /// Read the controller type ID register from the extension controller
    pub(super) async fn read_id(&mut self) -> Result<ControllerIdReport, AsyncImplError> {
        self.set_read_register_address(0xfa).await?;
        let i2c_id = self.read_ext_report().await?;
        Ok(i2c_id)
    }

    /// Determine the controller type based on the type ID of the extension controller
    pub(super) async fn identify_controller(
        &mut self,
    ) -> Result<Option<ControllerType>, AsyncImplError> {
        let i2c_id = self.read_id().await?;
        Ok(crate::core::identify_controller(i2c_id))
    }

    /// Instruct the extension controller to start preparing a sample by setting the read cursor to 0
    pub(super) async fn start_sample(&mut self) -> Result<(), AsyncImplError> {
        self.set_read_register_address(0x00).await?;
        Ok(())
    }
}
