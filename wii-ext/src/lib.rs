#![cfg_attr(not(test), no_std)]

/// Async I2C implementations
pub mod async_impl;

/// Blocking I2C implementations
pub mod blocking_impl;
/// Types + data decoding
pub mod core;
