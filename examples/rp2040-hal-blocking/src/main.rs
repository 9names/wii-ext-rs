//! Interact with a Wii extension controller via the wii-ext crate on a Pico board
//!
//! It will enumerate as a USB joystick, which you can use to control a game
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use bsp::hal::{
    self,
    clocks::{init_clocks_and_plls, Clock},
    entry,
    gpio,
    pac,
    sio::Sio,
    Timer,
    watchdog::Watchdog,
};
use fugit::RateExtU32;
use rp_pico as bsp;
use wii_ext::{classic_sync::Classic, core::classic::ClassicReadingCalibrated};

// use embedded_hal::Delay::delay_ms;
// use usb_device::class_prelude::*;
// use usb_device::prelude::*;
// use usbd_human_interface_device::device::joystick::JoystickReport;
// use usbd_human_interface_device::prelude::*;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
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

    // let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut delay = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
    //     pac.USBCTRL_REGS,
    //     pac.USBCTRL_DPRAM,
    //     clocks.usb_clock,
    //     true,
    //     &mut pac.RESETS,
    // ));

    // let mut joy = UsbHidClassBuilder::new()
    //     .add_device(usbd_human_interface_device::device::joystick::JoystickConfig::default())
    //     .build(&usb_bus);

    // //https://pid.codes
    // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
    //     .manufacturer("usbd-human-interface-device")
    //     .product("Rusty joystick")
    //     .serial_number("TEST")
    //     .build();

    let sda_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio8.reconfigure();
    let scl_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio9.reconfigure();

    let i2c = bsp::hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        100.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    // Create, initialise and calibrate the controller
    let mut controller = Classic::new(i2c, &mut delay).unwrap();

    let hi_res = false;

    // Enable hi-resolution mode. This also updates calibration
    // Don't really need it for this single stick mode. Plus it might make recovery easier...
    if hi_res {
        controller.enable_hires(&mut delay).unwrap();
    }

    // If you have a Nunchuk controller, use this instead.
    // let mut controller = Nunchuk::new(i2c, &mut delay).unwrap();
    loop {
        // Some controllers need a delay between reads or they become unhappy
        // delay.delay_ms(10);

        // Capture the current button and axis values
        let input = controller.read_blocking(&mut delay);
        if let Ok(input) = input {
            // match joy.device().write_report(&get_report(&input)) {
            //     Err(UsbHidError::WouldBlock) => {}
            //     Ok(_) => {}
            //     Err(e) => {
            //         core::panic!("Failed to write joystick report: {:?}", e)
            //     }
            // }
            // Print inputs from the controller
            debug!("{:?}", input);
        } else {
            // re-init controller on failure
            let _ = controller.init(&mut delay);
            if hi_res {
                let _ = controller.enable_hires(&mut delay);
            }
        }

        // if usb_dev.poll(&mut [&mut joy]) {}
    }
}

// fn get_report(input: &ClassicReadingCalibrated) -> JoystickReport {
//     // Read out buttons first
//     let mut buttons = 0;

//     buttons += input.button_b as u8;
//     buttons += (input.button_a as u8) << 1;
//     buttons += (input.button_y as u8) << 2;
//     buttons += (input.button_x as u8) << 3;
//     buttons += (input.button_trigger_l as u8) << 4;
//     buttons += (input.button_trigger_r as u8) << 5;
//     buttons += (input.button_minus as u8) << 6;
//     buttons += (input.button_plus as u8) << 7;

//     JoystickReport {
//         buttons,
//         x: input.joystick_left_x,
//         y: -input.joystick_left_y,
//     }
// }

// End of file
