use embedded_hal_mock::delay::MockNoop;
use embedded_hal_mock::i2c::{self, Transaction};
use paste::paste;
use wii_ext::classic::*;
use wii_ext::*;

/// There's a certain amount of slop around the center position.
/// Allow up to this range without it being an error
const ZERO_SLOP: i8 = 5;
/// Triggers are sloppier, or I accidentally pressed them during testing
const TRIGGER_SLOP: i8 = 10;
/// The max value at full deflection is ~100, but allow a bit less than that
const AXIS_MAX: i8 = 90;

macro_rules! assert_joystick_hd {
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
             fn [<test_calibrated_hd_ $y:lower>]()  {
                let expectations = vec![
                    // Reset controller
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    // Init
                    Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
                    Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),

                    // Calibration read (discarded - use any data)
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::PDP_LINK_IDLE.to_vec()),

                    // Switch to HD mode
                    Transaction::write(EXT_I2C_ADDR as u8, vec![254, 3]),

                    // HD-Mode Calibration read
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::$x.to_vec()),
                    // Input read
                    Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
                    Transaction::read(EXT_I2C_ADDR as u8, test_data::$y.to_vec()),
                ];
                let i2c = i2c::Mock::new(&expectations);
                let mut delay = MockNoop::new();
                let mut classic = Classic::new(i2c, &mut delay).unwrap();
                classic.enable_hires(&mut delay).unwrap();
                let input = classic.read_blocking(&mut delay).unwrap();

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
            }
        }
    };
}
// HD versions of the classic controller tests
// Left joystick moves left
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_LJOY_L, // Set idle and test sample
    i8::MIN, -AXIS_MAX, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves right
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_LJOY_R, // Set idle and test sample
    AXIS_MAX, i8::MAX, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves down
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_LJOY_D, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    i8::MIN, -AXIS_MAX, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Left joystick moves up
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_LJOY_U, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    AXIS_MAX, i8::MAX, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves left
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_RJOY_L, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    i8::MIN, -AXIS_MAX, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves right
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_RJOY_R, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    AXIS_MAX, i8::MAX, // acceptable range for right x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves down
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_RJOY_D, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    i8::MIN, -AXIS_MAX, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);

// Right joystick moves up
#[rustfmt::skip]
assert_joystick_hd!(
    PDP_LINK_HD_IDLE, PDP_LINK_HD_RJOY_U, // Set idle and test sample
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left x axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for left y axis
    -ZERO_SLOP, ZERO_SLOP, // acceptable range for right x axis
    AXIS_MAX, i8::MAX, // acceptable range for right y axis
    -TRIGGER_SLOP, TRIGGER_SLOP, // acceptable range for left trigger
    -TRIGGER_SLOP, TRIGGER_SLOP // // acceptable range for right trigger
);
