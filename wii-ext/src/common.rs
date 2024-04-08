/// Standard input report
pub type ExtReport = [u8; 6];
/// HD input report
pub type ExtHdReport = [u8; 8];
/// Controller ID report
pub type ControllerIdReport = [u8; 6];

#[cfg_attr(feature = "defmt_print", derive(defmt::Format))]
#[derive(Debug)]
pub enum ControllerType {
    Nunchuk,
    Classic,
    ClassicPro,
}

/// All Wii extension controllers use i2c address 52
pub const EXT_I2C_ADDR: u16 = 0x52;

/// There needs to be some time between i2c messages or the
/// wii ext device will abort the i2c transaction
/// 200 microseconds works in my tests - need to test with more devices
pub const INTERMESSAGE_DELAY_MICROSEC: u16 = 200;
pub const INTERMESSAGE_DELAY_MICROSEC_U32: u32 = 200;

pub fn identify_controller(id: ControllerIdReport) -> Option<ControllerType> {
    if id[2] != 0xA4 || id[3] != 0x20 {
        // Not an extension controller
        None
    } else if id[0] == 0 && id[1] == 0 && id[4] == 0 && id[5] == 0 {
        // It's a nunchuck
        Some(ControllerType::Nunchuk)
    } else if id[0] == 0 && id[1] == 0 && id[4] == 3 && id[5] == 1 {
        // It's a wii classic controller
        Some(ControllerType::Classic)
    } else if id[0] == 1 && id[1] == 0 && id[4] == 1 && id[5] == 1 {
        // It's a wii classic pro (or compatible) controller
        // This is most wii classic extension controllers (NES/SNES/Clones)
        Some(ControllerType::ClassicPro)
    } else {
        None
    }
}
