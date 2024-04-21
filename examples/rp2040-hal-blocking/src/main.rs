//! Interact with a Wii extension controller via the wii-ext crate on a Pico board
//!
//! It will enumerate as a USB joystick, which you can use to control a game
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use bsp::hal::{
    self, clocks::init_clocks_and_plls, entry, gpio, pac, sio::Sio, watchdog::Watchdog, Timer,
};
use embedded_hal::delay::DelayNs;
use fugit::RateExtU32;
use rp_pico as bsp;
use wii_ext::blocking_impl::classic::Classic;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sda_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio8.reconfigure();
    let scl_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio9.reconfigure();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        100.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    // Create, initialise and calibrate the controller
    let mut controller = Classic::new(i2c, delay).unwrap();

    let hi_res = false;

    // Enable hi-resolution mode. This also updates calibration
    // Don't really need it for this single stick mode. Plus it might make recovery easier...
    if hi_res {
        controller.enable_hires().unwrap();
    }

    // If you have a Nunchuk controller, use this instead.
    // let mut controller = Nunchuk::new(i2c, &mut delay).unwrap();
    loop {
        // Some controllers need a delay between reads or they become unhappy
        delay.delay_ms(10);

        // Capture the current button and axis values
        let input = controller.read();
        if let Ok(input) = input {
            // Print inputs from the controller
            debug!("{:?}", input);
        } else {
            // re-init controller on failure
            let _ = controller.init();
            if hi_res {
                let _ = controller.enable_hires();
            }
        }
    }
}

// End of file
