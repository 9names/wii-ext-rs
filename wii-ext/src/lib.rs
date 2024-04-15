#![cfg_attr(not(test), no_std)]

/// Blocking I2C impl
pub mod classic_sync;

// Async I2C impl
pub mod classic_async;

/// Types + data decoding
pub mod core;

/// i2c interface code
pub mod interface;
/// async i2c interface code
pub mod interface_async;

pub mod nunchuk;
