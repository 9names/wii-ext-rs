#![cfg_attr(not(test), no_std)]

/// async I2C impl
pub mod async_impl;
// async classic controller code
pub use async_impl::classic as classic_async;
/// async i2c interface code
pub use async_impl::interface as interface_async;
/// Blocking I2C impl
pub mod blocking_impl;
// blocking classic controller code
pub use blocking_impl::classic as classic_sync;
/// i2c interface code
pub use blocking_impl::interface;
// blocking nunchuk code
pub use blocking_impl::nunchuk;
/// Types + data decoding
pub mod core;
