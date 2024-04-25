# Rust Wiimote Extension-Controller (Nunchuk/classic controller) Driver

This is a platform agnostic Rust driver for Wiimote Extension controllers (Nunchuk, Classic, Classic Pro, NES Classic, SNES Classic, and clones) using the [`embedded-hal`] and [`embedded-hal-async`] traits.

This driver allows you to read all axes and buttons for Wiimote Extension controllers

Driver docs are available in [the wii-ext directory](wii-ext/README.md)


## Status

- Nunchuk is supported
- Classic controllers supported in regular and HD mode
- Controller init is not 100% reliable, can suffer from i2c errors. This seems to affect the blocking implementation more than async.  
  Error handling around new() is strongly recommended.

## Support

For questions, issues, feature requests like compatibility with other Wiimote extension controllers please file an
[issue in the github project](https://github.com/9names/wii-ext-rs/issues).

## License

Nunchuk portions of this crate are largely derived from  
<https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs>  
Copyright 2015, Paul Osborne <osbpau@gmail.com>

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   <http://opensource.org/licenses/MIT>)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async