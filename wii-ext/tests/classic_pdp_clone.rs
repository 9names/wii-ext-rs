use embedded_hal_mock::eh1::delay::NoopDelay;
use embedded_hal_mock::eh1::i2c::{self, Transaction};
use paste::paste;
use wii_ext::classic_sync::*;
use wii_ext::core::classic::ClassicReading;
use wii_ext::core::EXT_I2C_ADDR;
mod common;
use common::test_data;
use common::test_data::*;

/// There's a certain amount of slop around the center position.
/// Allow up to this range without it being an error
const ZERO_SLOP: i8 = 5;
/// Triggers are sloppier, or I accidentally pressed them during testing
const TRIGGER_SLOP: i8 = 25;
/// The max value at full deflection is ~100, but allow a bit less than that
const AXIS_MAX: i8 = 90;

/// The max value for the right stick is greatly reduced?
/// Need to retest in hi-resolution
const R_AXIS_MAX: i8 = 45;

fn assert_digital_eq(first: ClassicReading, other: ClassicReading) {
    assert_eq!(first.button_a, other.button_a);
    assert_eq!(first.button_b, other.button_b);
    assert_eq!(first.button_x, other.button_x);
    assert_eq!(first.button_y, other.button_y);
    assert_eq!(first.button_trigger_l, other.button_trigger_l);
    assert_eq!(first.button_trigger_r, other.button_trigger_r);
    assert_eq!(first.button_zl, other.button_zl);
    assert_eq!(first.button_zr, other.button_zr);
    assert_eq!(first.button_home, other.button_home);
    assert_eq!(first.button_plus, other.button_plus);
    assert_eq!(first.button_minus, other.button_minus);
}

/// Test that no buttons are pressed when the controller is idle
#[test]
fn classic_idle() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
    ];

    let mut i2c = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut classic = Classic::new(i2c.clone(), delay).unwrap();
    let report = classic.read_report_blocking().unwrap();
    assert_digital_eq(report, ClassicReading::default());
    i2c.done();
}

// We don't want to write all that out for every digital button, so let's write a macro instead.
// Here's what it would look like to test that button a is the only thing pressed in the
// PDP_LINK_BTN_A test data:

// assert_button_fn!(button_a, PDP_LINK_BTN_A);

// yields

// #[test]
// fn test_button_a_on_classic_btn_a() {
//     let expectations = vec![
//         Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
//         Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_BTN_A.to_vec()),
//     ];
//     let mock = i2c::Mock::new(&expectations);
//     let mut nc = Classic { i2cdev: mock };
//     let report = nc.read_raw_no_wait().unwrap();
//     report.assert_digital_eq(ClassicReading {
//         button_a: true,
//         ..Default::default()
//     });
// }

macro_rules! assert_button_fn {
    ( $x:ident, $y:ident ) => {
        paste! {
            #[test]
                fn [<test_ $x _on_ $y:lower>]()  {
                let expectations = vec![
                    // Reset controller
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    // Init
                    Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
                    Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
                    // Read
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, $y.to_vec()),
                ];
                let mut i2c = i2c::Mock::new(&expectations);
                let delay = NoopDelay::new();
                let mut classic = Classic::new(i2c.clone(), delay).unwrap();
                let input = classic.read_report_blocking().unwrap();
                assert_digital_eq(input, ClassicReading {
                    $x: true,
                    ..Default::default()
                });
                i2c.done();
            }
        }
    };
}

// Test all the digital inputs for the original classic controller
assert_button_fn!(dpad_up, PDP_LINK_PAD_U);
assert_button_fn!(dpad_down, PDP_LINK_PAD_D);
assert_button_fn!(dpad_left, PDP_LINK_PAD_L);
assert_button_fn!(dpad_right, PDP_LINK_PAD_R);
assert_button_fn!(button_b, PDP_LINK_BTN_B);
assert_button_fn!(button_a, PDP_LINK_BTN_A);
assert_button_fn!(button_x, PDP_LINK_BTN_X);
assert_button_fn!(button_y, PDP_LINK_BTN_Y);
assert_button_fn!(button_trigger_l, PDP_LINK_BTN_L);
assert_button_fn!(button_trigger_r, PDP_LINK_BTN_R);
assert_button_fn!(button_zl, PDP_LINK_BTN_ZL);
assert_button_fn!(button_zr, PDP_LINK_BTN_ZR);
assert_button_fn!(button_minus, PDP_LINK_BTN_MINUS);
assert_button_fn!(button_plus, PDP_LINK_BTN_PLUS);
assert_button_fn!(button_home, PDP_LINK_BTN_HOME);

/// Test that no buttons are pressed when the controller is idle
#[test]
fn classic_calibrated_idle() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
        // Input read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
    ];
    let mut i2c = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut classic = Classic::new(i2c.clone(), delay).unwrap();
    let input = classic.read_blocking().unwrap();
    assert_eq!(input.joystick_left_x, 0);
    assert_eq!(input.joystick_left_y, 0);
    assert_eq!(input.joystick_right_x, 0);
    assert_eq!(input.joystick_right_y, 0);
    i2c.done();
}

/// Test that no buttons are pressed when the controller is idle
#[test]
fn classic_calibrated_joy_left() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),
        // Input readtest_data
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_LJOY_L.to_vec()),
    ];
    let mut i2c = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut classic = Classic::new(i2c.clone(), delay).unwrap();
    let input = classic.read_blocking().unwrap();

    assert!(
        (i8::MIN..-AXIS_MAX).contains(&input.joystick_left_x),
        "left_x = {}",
        input.joystick_left_x
    );
    assert!(
        (-ZERO_SLOP..ZERO_SLOP).contains(&input.joystick_left_y),
        "left_y = {}",
        input.joystick_left_y
    );
    assert!(
        (-ZERO_SLOP..ZERO_SLOP).contains(&input.joystick_right_x),
        "right_x = {}",
        input.joystick_right_x
    );
    assert!(
        (-ZERO_SLOP..ZERO_SLOP).contains(&input.joystick_right_y),
        "right_y = {}",
        input.joystick_right_y
    );
    assert!(
        (-TRIGGER_SLOP..TRIGGER_SLOP).contains(&input.trigger_left),
        "trigger_left = {}",
        input.trigger_left
    );
    assert!(
        (-TRIGGER_SLOP..TRIGGER_SLOP).contains(&input.trigger_right),
        "trigger_right = {}",
        input.trigger_right
    );
    i2c.done();
}

macro_rules! assert_joysticks {
    ( $x:ident, $y:ident,
        $lxl:expr, $lxh:expr,
        $lyl:expr, $lyh:expr,
        $rxl:expr, $rxh:expr,
        $ryl:expr, $ryh:expr,
        $ltl:expr, $lth:expr,
        $rtl:expr, $rth:expr
    ) => {
        paste! {
            #[test]
                fn [<test_calibrated_ $y:lower>]()  {
                let expectations = vec![
                    // Reset controller
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    // Init
                    Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
                    Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
                    // Calibration read
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::$x.to_vec()),
                    // Input read
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::$y.to_vec()),
                ];
                let mut i2c = i2c::Mock::new(&expectations);
                let delay = NoopDelay::new();
                let mut classic = Classic::new(i2c.clone(), delay).unwrap();
                let input = classic.read_blocking().unwrap();

                assert!(
                    ($lxl..=$lxh).contains(&input.joystick_left_x),
                    "left_x = {}, expected between {} and {}",
                    input.joystick_left_x,
                    $lxl,
                    $lxh
                );
                assert!(
                    ($lyl..=$lyh).contains(&input.joystick_left_y),
                    "left_y = {}, expected between {} and {}",
                    input.joystick_left_y,
                    $lyl,
                    $lyh
                );
                assert!(
                    ($rxl..=$rxh).contains(&input.joystick_right_x),
                    "right_x = {}, expected between {} and {}",
                    input.joystick_right_x,
                    $rxl,
                    $rxh
                );
                assert!(
                    ($ryl..=$ryh).contains(&input.joystick_right_y),
                    "right_y = {}, expected between {} and {}",
                    input.joystick_right_y,
                    $ryl,
                    $ryh
                );
                assert!(
                    ($ltl..=$lth).contains(&input.trigger_left),
                    "trigger_left = {}, expected between {} and {}",
                    input.trigger_left,
                    $ltl,
                    $lth
                );
                assert!(
                    ($rtl..=$rth).contains(&input.trigger_right),
                    "trigger_right = {}, expected between {} and {}",
                    input.trigger_right,
                    $rtl,
                    $rth
                );
                i2c.done();
            }
        }
    };
}

// This is the equivalent of classic_calibrated_joy_left
// Left joystick moves left
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_LJOY_L, // Set idle and test sample
    i8::MIN, -AXIS_MAX, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves right
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_LJOY_R, // Set idle and test sample
    AXIS_MAX, i8::MAX, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves down
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_LJOY_D, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    i8::MIN, -AXIS_MAX, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves up
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_LJOY_U, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    AXIS_MAX, i8::MAX, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves left
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_RJOY_L, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    i8::MIN, -AXIS_MAX, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves right
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_RJOY_R, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    AXIS_MAX, i8::MAX, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves down
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_RJOY_D, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    i8::MIN, -R_AXIS_MAX, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves up
#[rustfmt::skip]
assert_joysticks!(
    PDP_LINK_IDLE, PDP_LINK_RJOY_U, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    R_AXIS_MAX, i8::MAX, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);
