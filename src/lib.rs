#![cfg_attr(not(test), no_std)]
pub mod classic;
pub mod nunchuk;

/// Standard input or ID report
pub type ExtReport = [u8; 6];
/// HD input report
pub type ExtHdReport = [u8; 8];

/// All Wii extension controllers use i2c address 52
pub const EXT_I2C_ADDR: u16 = 52;

#[cfg(test)]
mod test_data;
