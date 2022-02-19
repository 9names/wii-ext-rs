#![cfg_attr(not(test), no_std)]
pub mod classic;
pub mod nunchuk;
/// Standard input or ID report
pub type ExtReport = [u8; 6];
/// HD input report
pub type ExtHdReport = [u8; 8];

/// All Wii extension controllers use i2c address 52
pub const EXT_I2C_ADDR: u16 = 0x52;

/// There needs to be some time between i2c messages or the
/// wii ext device will abort the i2c transaction
/// 200 microseconds works in my tests - need to test with more devices
pub const INTERMESSAGE_DELAY_MICROSEC: u16 = 200;

/// Test data used by the integration tests to confirm that the driver is working.  
/// Not intended for use outside of this crate.
pub mod test_data;
