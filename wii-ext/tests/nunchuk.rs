use embedded_hal_mock::eh1::{
    delay::NoopDelay,
    i2c::{self, Transaction},
};
use wii_ext::{nunchuk::Nunchuk, test_data, EXT_I2C_ADDR};
/// There's a certain amount of slop around the center position.
/// Allow up to this range without it being an error
const ZERO_SLOP: i8 = 5;
/// The max value at full deflection is ~100, but allow a bit less than that
const AXIS_MAX: i8 = 90;

// TODO: work out how to test analogue values from joystick and gyro

#[test]
fn nunchuck_idle() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
    ];

    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    mock.done();
}

#[test]
fn nunchuck_idle_calibrated() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    assert_eq!(report.joystick_x, 0);
    assert_eq!(report.joystick_y, 0);
    mock.done();
}

#[test]
fn nunchuck_left_calibrated() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_L.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    assert!(report.joystick_x < -AXIS_MAX, "x = {}", report.joystick_x);
    assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
    assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
    mock.done();
}

#[test]
fn nunchuck_right_calibrated() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_R.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    assert!(report.joystick_x > AXIS_MAX, "x = {}", report.joystick_x);
    assert!(report.joystick_y > -ZERO_SLOP, "y = {}", report.joystick_y);
    assert!(report.joystick_y < ZERO_SLOP, "y = {}", report.joystick_y);
    mock.done();
}

#[test]
fn nunchuck_up_calibrated() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_U.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    assert!(report.joystick_y > AXIS_MAX, "y = {}", report.joystick_y);
    assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
    assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
    mock.done();
}

#[test]
fn nunchuck_down_calibrated() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_JOY_D.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();
    let report = nc.read_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    assert!(report.joystick_y < -AXIS_MAX, "y = {}", report.joystick_y);
    assert!(report.joystick_x > -ZERO_SLOP, "x = {}", report.joystick_x);
    assert!(report.joystick_x < ZERO_SLOP, "x = {}", report.joystick_x);
    mock.done();
}

#[test]
fn nunchuck_idle_repeat() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

    let report = nc.read_report_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    let report = nc.read_report_blocking().unwrap();
    assert!(!report.button_c);
    assert!(!report.button_z);
    mock.done();
}

#[test]
fn nunchuck_btn_c() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_C.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

    let report = nc.read_report_blocking().unwrap();
    assert!(report.button_c);
    assert!(!report.button_z);
    mock.done();
}

#[test]
fn nunchuck_btn_z() {
    let expectations = vec![
        // Reset controller
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        // Init
        Transaction::write(EXT_I2C_ADDR as u8, vec![240, 85]),
        Transaction::write(EXT_I2C_ADDR as u8, vec![251, 0]),
        // Calibration read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        // Read
        Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
        Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_Z.to_vec()),
    ];
    let mut mock = i2c::Mock::new(&expectations);
    let delay = NoopDelay::new();
    let mut nc = Nunchuk::new(mock.clone(), delay).unwrap();

    let report = nc.read_report_blocking().unwrap();
    assert!(!report.button_c);
    assert!(report.button_z);
    mock.done();
}
