/// Data from a classic controller after it has been deserialized
///
/// In low-res mode, axes with less than 8 bits of range will be
/// scaled to approximate an 8 bit range.
/// in hi-res mode, all axes arleady have 8 bits of range
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct ClassicReading {
    pub joystick_left_x: u8,
    pub joystick_left_y: u8,
    pub joystick_right_x: u8,
    pub joystick_right_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_b: bool,
    pub button_a: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_trigger_l: bool,
    pub button_trigger_r: bool,
    pub button_zl: bool,
    pub button_zr: bool,
    pub button_minus: bool,
    pub button_plus: bool,
    pub button_home: bool,
}

/// Data from a classic controller after calibration data has been applied
///
/// Calibration is done by subtracting the resting values from the current
/// values, which means that going lower on the axis will go negative.
/// Due to this, we now store analog values as signed integers
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct ClassicReadingCalibrated {
    pub joystick_left_x: i8,
    pub joystick_left_y: i8,
    pub joystick_right_x: i8,
    pub joystick_right_y: i8,
    pub trigger_left: i8,
    pub trigger_right: i8,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_b: bool,
    pub button_a: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_trigger_l: bool,
    pub button_trigger_r: bool,
    pub button_zl: bool,
    pub button_zr: bool,
    pub button_minus: bool,
    pub button_plus: bool,
    pub button_home: bool,
}

impl ClassicReadingCalibrated {
    pub fn new(r: ClassicReading, c: &CalibrationData) -> ClassicReadingCalibrated {
        /// Just in case `data` minus `calibration data` is out of range, perform all operations
        /// on i16 and clamp to i8 limits before returning
        fn ext_u8_sub(a: u8, b: u8) -> i8 {
            let res = (a as i16) - (b as i16);
            res.clamp(i8::MIN as i16, i8::MAX as i16) as i8
        }

        ClassicReadingCalibrated {
            joystick_left_x: ext_u8_sub(r.joystick_left_x, c.joystick_left_x),
            joystick_left_y: ext_u8_sub(r.joystick_left_y, c.joystick_left_y),
            joystick_right_x: ext_u8_sub(r.joystick_right_x, c.joystick_right_x),
            joystick_right_y: ext_u8_sub(r.joystick_right_y, c.joystick_right_y),
            trigger_left: ext_u8_sub(r.trigger_left, c.trigger_left),
            trigger_right: ext_u8_sub(r.trigger_right, c.trigger_right),
            dpad_up: r.dpad_up,
            dpad_down: r.dpad_down,
            dpad_left: r.dpad_left,
            dpad_right: r.dpad_right,
            button_b: r.button_b,
            button_a: r.button_a,
            button_x: r.button_x,
            button_y: r.button_y,
            button_trigger_l: r.button_trigger_l,
            button_trigger_r: r.button_trigger_r,
            button_zl: r.button_zl,
            button_zr: r.button_zr,
            button_minus: r.button_minus,
            button_plus: r.button_plus,
            button_home: r.button_home,
        }
    }
}

/// Convert raw data as returned from controller via i2c into buttons and axis fields
#[rustfmt::skip]
pub(crate) fn decode_classic_report(data: &[u8]) -> ClassicReading {
    // Classic mode:
    //  Bit	7	6	5	4	3	2	1	0
    // 	Byte
    // 	0	RX<4:3>	LX<5:0>
    // 	1	RX<2:1>	LY<5:0>
    // 	2	RX<0>	LT<4:3>	RY<4:0>
    // 	3	LT<2:0>	RT<4:0>
    // 	4	BDR	BDD	BLT	B-	BH	B+	BRT	1
    // 	5	BZL	BB	BY	BA	BX	BZR	BDL	BDU
    ClassicReading {
        joystick_left_x:   ClassicReading::scale_6bit_8bit(data[0] & 0b0011_1111),
        joystick_left_y:   ClassicReading::scale_6bit_8bit(data[1] & 0b0011_1111),
        joystick_right_x:  ClassicReading::scale_5bit_8bit(
            ((data[2] & 0b1000_0000) >> 7) |
            ((data[1] & 0b1100_0000) >> 5) |
            ((data[0] & 0b1100_0000) >> 3)
        ),
        joystick_right_y:  ClassicReading::scale_5bit_8bit(data[2] & 0b0001_1111),
        trigger_left:     ClassicReading::scale_5bit_8bit(
            ((data[2] & 0b0110_0000) >> 2) |
            ((data[3] & 0b1110_0000) >> 5)
        ),
        trigger_right:    ClassicReading::scale_5bit_8bit(data[3] & 0b0001_1111),
        dpad_right:       data[4] & 0b1000_0000 == 0,
        dpad_down:        data[4] & 0b0100_0000 == 0,
        button_trigger_l: data[4] & 0b0010_0000 == 0,
        button_minus:     data[4] & 0b0001_0000 == 0,
        button_home:      data[4] & 0b0000_1000 == 0,
        button_plus:      data[4] & 0b0000_0100 == 0,
        button_trigger_r: data[4] & 0b0000_0010 == 0,
        button_zl:        data[5] & 0b1000_0000 == 0,
        button_b:         data[5] & 0b0100_0000 == 0,
        button_y:         data[5] & 0b0010_0000 == 0,
        button_a:         data[5] & 0b0001_0000 == 0,
        button_x:         data[5] & 0b0000_1000 == 0,
        button_zr:        data[5] & 0b0000_0100 == 0,
        dpad_left:        data[5] & 0b0000_0010 == 0,
        dpad_up:          data[5] & 0b0000_0001 == 0,
    }
}

/// Convert high-resolution raw data as returned from controller via i2c into buttons and axis fields
#[rustfmt::skip]
pub(crate) fn decode_classic_hd_report(data: &[u8]) -> ClassicReading {
    // High precision mode:
    // Bit    7    6    5    4    3    2    1    0
    // Byte
    // 0      LX<7:0>
    // 1      RX<7:0>
    // 2      LY<7:0>
    // 3      RY<7:0>
    // 4      LT<7:0>
    // 5      RT<7:0>
    // 6      BDR  BDD  BLT  B-   BH   B+   BRT  1
    // 7      BZL  BB   BY   BA   BX   BZR  BDL  BDU
    ClassicReading {
        joystick_left_x:   data[0],
        joystick_right_x:  data[1],
        joystick_left_y:   data[2],
        joystick_right_y:  data[3],
        trigger_left:     data[4],
        trigger_right:    data[5],
        dpad_right:       data[6] & 0b1000_0000 == 0,
        dpad_down:        data[6] & 0b0100_0000 == 0,
        button_trigger_l: data[6] & 0b0010_0000 == 0,
        button_minus:     data[6] & 0b0001_0000 == 0,
        button_home:      data[6] & 0b0000_1000 == 0,
        button_plus:      data[6] & 0b0000_0100 == 0,
        button_trigger_r: data[6] & 0b0000_0010 == 0,
        button_zl:        data[7] & 0b1000_0000 == 0,
        button_b:         data[7] & 0b0100_0000 == 0,
        button_y:         data[7] & 0b0010_0000 == 0,
        button_a:         data[7] & 0b0001_0000 == 0,
        button_x:         data[7] & 0b0000_1000 == 0,
        button_zr:        data[7] & 0b0000_0100 == 0,
        dpad_left:        data[7] & 0b0000_0010 == 0,
        dpad_up:          data[7] & 0b0000_0001 == 0,
    }
}

/// Relaxed/Center positions for each axis
///
/// These are used to calculate the relative deflection of each access from their center point
#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct CalibrationData {
    pub joystick_left_x: u8,
    pub joystick_left_y: u8,
    pub joystick_right_x: u8,
    pub joystick_right_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
}

impl ClassicReading {
    #[cfg(test)]
    /// Helper function for testing digital pin status
    /// This should work for all different classic controllers
    /// Testing analogue is harder, will have to think about testing those.
    pub fn assert_digital_eq(&self, other: ClassicReading) {
        assert_eq!(self.button_a, other.button_a);
        assert_eq!(self.button_b, other.button_b);
        assert_eq!(self.button_x, other.button_x);
        assert_eq!(self.button_y, other.button_y);
        assert_eq!(self.button_trigger_l, other.button_trigger_l);
        assert_eq!(self.button_trigger_r, other.button_trigger_r);
        assert_eq!(self.button_zl, other.button_zl);
        assert_eq!(self.button_zr, other.button_zr);
        assert_eq!(self.button_home, other.button_home);
        assert_eq!(self.button_plus, other.button_plus);
        assert_eq!(self.button_minus, other.button_minus);
    }

    /// Some axis' data is u5, scale it to u8 for convenience
    pub(crate) fn scale_5bit_8bit(reading: u8) -> u8 {
        // TODO: better math here, move this somewhere common
        ((reading as u32 * u8::MAX as u32) / 31) as u8
    }

    /// Some axis' data is u6, scale it to u8 for convenience
    pub(crate) fn scale_6bit_8bit(reading: u8) -> u8 {
        // TODO: better math here, move this somewhere common
        ((reading as u32 * u8::MAX as u32) / 63) as u8
    }

    /// Convert from a wii-ext report into controller data
    pub fn from_data(data: &[u8]) -> Option<ClassicReading> {
        if data.len() == 6 {
            // Classic mode:
            Some(decode_classic_report(data))
        } else if data.len() == 8 {
            // High precision mode:
            Some(decode_classic_hd_report(data))
        } else {
            None
        }
    }
}
