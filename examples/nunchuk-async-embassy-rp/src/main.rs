#![no_std]
#![no_main]

use defmt::*;
use embassy_rp::gpio;
use gpio::{Level, Output};
use wii_ext::async_impl::nunchuk::Nunchuk;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::AnyPin;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C0;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Duration, Ticker};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

type LedType = Mutex<ThreadModeRawMutex, Option<Output<'static, AnyPin>>>;
static LED: LedType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");
    let p = embassy_rp::init(Default::default());

    // Configure Pico LED to blink once a second
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(led);
    }
    unwrap!(spawner.spawn(toggle_led(&LED, Duration::from_millis(500))));

    let sda = p.PIN_8;
    let scl = p.PIN_9;

    info!("set up i2c");
    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    // Create, initialise and calibrate the controller
    info!("initialising controller");
    let mut controller = Nunchuk::new(i2c, Delay);
    controller.init().await.unwrap();

    info!("begin polling controller");
    loop {
        let input = controller.read().await.unwrap();
        debug!("{:?}", input);
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn toggle_led(led: &'static LedType, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    loop {
        {
            let mut led_unlocked = led.lock().await;
            if let Some(pin_ref) = led_unlocked.as_mut() {
                pin_ref.toggle();
            }
        }
        ticker.next().await;
    }
}

#[cortex_m_rt::pre_init]
unsafe fn before_main() {
    // Soft-reset doesn't clear spinlocks. Clear the one used by critical-section
    // before we hit main to avoid deadlocks when using a debugger
    embassy_rp::pac::SIO.spinlock(31).write_value(1);
}
