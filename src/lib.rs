#![cfg_attr(not(test), no_std)]
/// Blocking I2C impl
pub mod classic;
/// Types + data decoding
pub mod classic_core;
/// Anything common between nunchuk + classic
pub mod common;

pub mod nunchuk;

/// Test data used by the integration tests to confirm that the driver is working.  
/// Not intended for use outside of this crate.
pub mod test_data;

// Expose all common types at the crate level
pub use common::*;
