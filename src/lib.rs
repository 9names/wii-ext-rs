#![cfg_attr(not(test), no_std)]
#![cfg_attr(
    feature = "async",
    feature(type_alias_impl_trait, inherent_associated_types)
)]
#[cfg(feature = "async")]
pub mod classic_async;
/// Blocking I2C impl
pub mod classic_sync;

/// Types + data decoding
pub mod core;

/// Anything common between nunchuk + classic
pub mod common;

pub mod nunchuk;

/// Test data used by the integration tests to confirm that the driver is working.  
/// Not intended for use outside of this crate.
pub mod test_data;

// Expose all common types at the crate level
pub use common::*;
