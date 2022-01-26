use crate::ExtReport;
use crate::EXT_I2C_ADDR;
use crate::INTERMESSAGE_DELAY_MICROSEC;
// nunchuk technically supports HD report, but the last two bytes will be zeroes
// TODO: work out if it's worth supporting that
// TODO: add support for the
// use crate::ExtHdReport;
use embedded_hal::blocking::delay::DelayUs;

#[cfg(feature = "defmt_print")]
use defmt;

#[derive(Debug)]
pub enum NunchukError<E> {
    Error(E),
    ParseError,
}

/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub struct NunchukReading {
    pub joystick_x: u8,
    pub joystick_y: u8,
    pub accel_x: u16, // 10-bit
    pub accel_y: u16, // 10-bit
    pub accel_z: u16, // 10-bit
    pub c_button_pressed: bool,
    pub z_button_pressed: bool,
}

impl NunchukReading {
    pub fn from_data(data: &[u8]) -> Option<NunchukReading> {
        if data.len() < 6 {
            None
        } else {
            Some(NunchukReading {
                joystick_x: data[0],
                joystick_y: data[1],
                accel_x: (u16::from(data[2]) << 2) | ((u16::from(data[5]) >> 6) & 0b11),
                accel_y: (u16::from(data[3]) << 2) | ((u16::from(data[5]) >> 4) & 0b11),
                accel_z: (u16::from(data[4]) << 2) | ((u16::from(data[5]) >> 2) & 0b11),
                c_button_pressed: (data[5] & 0b10) == 0,
                z_button_pressed: (data[5] & 0b01) == 0,
            })
        }
    }
}

pub struct Nunchuk<I2C> {
    i2cdev: I2C,
}
use embedded_hal::blocking::i2c as i2ctrait;
impl<T, E> Nunchuk<T>
where
    T: i2ctrait::Write<Error = E> + i2ctrait::Read<Error = E> + i2ctrait::WriteRead<Error = E>,
{
    /// Create a new Wii Nunchuk
    ///
    /// This method will open the provide i2c device file and will
    /// send the required init sequence in order to read data in
    /// the future.
    pub fn new<D: DelayUs<u16>>(i2cdev: T, delay: &mut D) -> Result<Nunchuk<T>, Error<E>> {
        let mut nunchuk = Nunchuk { i2cdev };
        nunchuk.init(delay)?;
        Ok(nunchuk)
    }

    fn set_read_register_address(&mut self, address: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[address])
            .map_err(Error::I2C)
    }

    fn set_register(&mut self, reg: u8, val: u8) -> Result<(), Error<E>> {
        self.i2cdev
            .write(EXT_I2C_ADDR as u8, &[reg, val])
            .map_err(Error::I2C)
    }

    fn read_report(&mut self) -> Result<ExtReport, Error<E>> {
        let mut buffer: ExtReport = ExtReport::default();
        self.i2cdev
            .read(EXT_I2C_ADDR as u8, &mut buffer)
            .map_err(Error::I2C)
            .and(Ok(buffer))
    }

    /// Send the init sequence to the Wii extension controller
    ///
    /// This could be a bit faster with DelayUs, but since you only init once we'll re-use delay_ms
    pub fn init<D: DelayUs<u16>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
        // These registers must be written to disable encryption.; the documentation is a bit
        // lacking but it appears this is some kind of handshake to
        // perform unencrypted data tranfers
        self.set_register(0xF0, 0x55)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.set_register(0xFB, 0x00)?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        Ok(())
    }

    /// tell the extension controller to prepare a sample by setting the read cursor to 0
    fn start_sample(&mut self) -> Result<(), Error<E>> {
        self.set_read_register_address(0x00)?;

        Ok(())
    }

    fn read_nunchuk(&mut self) -> Result<NunchukReading, Error<E>> {
        let buf = self.read_report()?;
        NunchukReading::from_data(&buf).ok_or(Error::InvalidInputData)
    }

    /// Simple helper with no delay. Should work for testing, not sure if it will function on hardware
    pub fn read_no_wait(&mut self) -> Result<NunchukReading, Error<E>> {
        self.start_sample()?;
        self.read_nunchuk()
    }

    /// Simple blocking read helper that will start a sample, wait 10ms, then read the value
    pub fn read_blocking<D: DelayUs<u16>>(
        &mut self,
        delay: &mut D,
    ) -> Result<NunchukReading, Error<E>> {
        self.start_sample()?;
        delay.delay_us(INTERMESSAGE_DELAY_MICROSEC);
        self.read_nunchuk()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data;
    use embedded_hal_mock::i2c::{self, Transaction};

    // TODO: work out how to test analogue values from joystick and gyro

    #[test]
    fn nunchuck_idle() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk { i2cdev: mock };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_idle_repeat() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_IDLE.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk { i2cdev: mock };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_btn_c() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_C.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk { i2cdev: mock };
        let report = nc.read_no_wait().unwrap();
        assert!(report.c_button_pressed);
        assert!(!report.z_button_pressed);
    }

    #[test]
    fn nunchuck_btn_z() {
        let expectations = vec![
            Transaction::write(EXT_I2C_ADDR as u8, vec![0]),
            Transaction::read(EXT_I2C_ADDR as u8, test_data::NUNCHUCK_BTN_Z.to_vec()),
        ];
        let mock = i2c::Mock::new(&expectations);
        let mut nc = Nunchuk { i2cdev: mock };
        let report = nc.read_no_wait().unwrap();
        assert!(!report.c_button_pressed);
        assert!(report.z_button_pressed);
    }
}
