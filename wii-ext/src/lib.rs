#![doc = include_str!("../../README.md")]
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
//
// The nunchuk portion of this crate is derived from
// https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs
// which is Copyright 2015, Paul Osborne <osbpau@gmail.com>
#![cfg_attr(not(test), no_std)]

/// Async I2C implementations
pub mod async_impl;

/// Blocking I2C implementations
pub mod blocking_impl;
/// Types + data decoding
pub mod core;
