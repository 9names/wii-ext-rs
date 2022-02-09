# Rust Wiimote Extension-Controller (Nunchuck/classic controller) Driver

This is a platform agnostic Rust driver for Wiimote Extension controllers (Nunchuk, Classic, Classic Pro, NES Classic, SNES Classic, and clones) using the [`embedded-hal`] traits.

This driver allows you to read all axes and buttons for Wiimote Extension controllers

## Physical protocol details

Wiimote extension controllers are designed to talk to a Wiimote over an I2C interface at 3.3V.
The official controllers are capable of operating in fast-mode (400Khz) though some clones require normal-mode (100Khz).
The protocol is quite simple - it's not officially documented, but it has been reverse-engineered.

- http://wiibrew.org/wiki/Wiimote/Extension_Controllers
- http://wiibrew.org/wiki/Wiimote/Extension_Controllers/Nunchuck
- http://wiibrew.org/wiki/Wiimote/Extension_Controllers/Classic_Controller

High Resolution mode is a recent addition and was only discovered once the NES Classic console was released. It is described here:
- https://www.raphnet-tech.com/support/classic_controller_high_res/


Wii Motion Plus support is planned, both in standalone and combo mode

## Usage

To use this driver, import this crate and an `embedded_hal` implementation,
then instantiate the appropriate device.

```rust
use wii_ext::classic::Classic;
use ::I2C; // insert an include for your HAL i2c peripheral name here

fn main() {
    let i2c = I2C::new(); // insert your HAL i2c init here
    let mut delay = cortex_m::delay::Delay::new(); // some delay source as well
    // Create, initialise and calibrate the controller
    let mut controller = Classic::new(i2c, &mut delay).unwrap();
    // Enable hi-resolution mode. This also updates calibration
    controller.enable_hires(&mut delay).unwrap();
    loop {
        let input = controller.read_blocking(&mut delay).unwrap();
        // You can read individual buttons...
        let a = input.button_a;
        let b = input.button_b;
        // or joystick axes
        let x = input.joysick_left_x;
        let y = input.joysick_left_y;
        // the data structs optionally support defmt::debug
        // if you enable features=["defmt_print"]
        info!("{:?}", read);
        // Calibration can be manually performed as needed
        controller.update_calibration();
    }
}
```

## Status

- Nunchuk is functional, no calibration yet
- Classic controllers supported in regular and HD mode

## Support

For questions, issues, feature requests like compatibility with other Wiimote extension controllers please file an
[issue in the github project](https://github.com/9names/wii-ext-rs/issues).

## License

Nunchuk portions of this crate are largely derived from  
https://github.com/rust-embedded/rust-i2cdev/blob/master/examples/nunchuck.rs  
Copyright 2015, Paul Osborne <osbpau@gmail.com>

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal