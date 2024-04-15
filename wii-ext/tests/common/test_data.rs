#![allow(dead_code)]

use wii_ext::core::ExtHdReport;
use wii_ext::core::ExtReport;
// Test data with each peripheral in certain states
// ID is the identification data from address
// CLASSIC_HIRES_DEFAULT is the value of the hi-res mode register *before* setting it
// since we'll need that to restore to regular mode

// Nunchuck
pub const NUNCHUCK_ID: ExtReport = [0, 0, 164, 32, 0, 0];
pub const NUNCHUCK_IDLE: ExtReport = [126, 129, 125, 139, 170, 95];
pub const NUNCHUCK_JOY_U: ExtReport = [130, 221, 125, 118, 172, 191];
pub const NUNCHUCK_JOY_D: ExtReport = [126, 35, 130, 131, 173, 7];
pub const NUNCHUCK_JOY_L: ExtReport = [25, 130, 117, 126, 172, 191];
pub const NUNCHUCK_JOY_R: ExtReport = [225, 130, 122, 132, 173, 27];
pub const NUNCHUCK_BTN_C: ExtReport = [127, 128, 122, 138, 171, 181];
pub const NUNCHUCK_BTN_Z: ExtReport = [127, 127, 122, 134, 172, 122];
pub const NUNCHUCK_HD_IDLE: ExtHdReport = [126, 128, 148, 119, 160, 211, 0, 0];

// NES classic controller
pub const NES_ID: ExtReport = [1, 0, 164, 32, 1, 1];
pub const NES_IDLE: ExtReport = [95, 223, 143, 0, 255, 255];
pub const NES_BTN_B: ExtReport = [95, 223, 143, 0, 255, 191];
pub const NES_BTN_A: ExtReport = [95, 223, 143, 0, 255, 239];
pub const NES_BTN_SELECT: ExtReport = [95, 223, 143, 0, 239, 255];
pub const NES_BTN_START: ExtReport = [95, 223, 143, 0, 251, 255];
pub const NES_PAD_U: ExtReport = [95, 223, 143, 0, 255, 254];
pub const NES_PAD_D: ExtReport = [95, 223, 143, 0, 191, 255];
pub const NES_PAD_L: ExtReport = [95, 223, 143, 0, 255, 253];
pub const NES_PAD_R: ExtReport = [95, 223, 143, 0, 127, 255];
pub const NES_HD_IDLE: ExtHdReport = [127, 127, 127, 127, 000, 000, 255, 255];

// SNES classic
pub const SNES_ID: ExtReport = [1, 0, 164, 32, 1, 1];
pub const SNES_IDLE: ExtReport = [160, 33, 16, 0, 255, 255];
pub const SNES_BTN_B: ExtReport = [160, 33, 16, 0, 255, 191];
pub const SNES_BTN_A: ExtReport = [95, 223, 143, 0, 255, 239];
pub const SNES_BTN_X: ExtReport = [160, 33, 16, 0, 255, 247];
pub const SNES_BTN_Y: ExtReport = [160, 33, 16, 0, 255, 223];
pub const SNES_BTN_L: ExtReport = [160, 33, 112, 224, 223, 255];
pub const SNES_BTN_R: ExtReport = [160, 33, 16, 31, 253, 255];
pub const SNES_PAD_U: ExtReport = [95, 223, 143, 0, 255, 254];
pub const SNES_PAD_D: ExtReport = [95, 223, 143, 0, 191, 255];
pub const SNES_PAD_L: ExtReport = [95, 223, 143, 0, 255, 253];
pub const SNES_PAD_R: ExtReport = [95, 223, 143, 0, 127, 255];
pub const SNES_BTN_SELECT: ExtReport = [95, 223, 143, 0, 239, 255];
pub const SNES_BTN_START: ExtReport = [95, 223, 143, 0, 251, 255];
pub const SNES_HD_IDLE: ExtHdReport = [128, 132, 132, 132, 0, 0, 255, 255];

// Wii Classic controller
pub const CLASSIC_ID: ExtReport = [0, 0, 164, 32, 1, 1];
pub const CLASSIC_HIRES_DEFAULT: u8 = 1;
pub const CLASSIC_IDLE: ExtReport = [97, 224, 145, 99, 255, 255];
pub const CLASSIC_BTN_B: ExtReport = [97, 224, 145, 99, 255, 191];
pub const CLASSIC_BTN_A: ExtReport = [97, 224, 145, 99, 255, 239];
pub const CLASSIC_BTN_X: ExtReport = [97, 224, 145, 99, 255, 247];
pub const CLASSIC_BTN_Y: ExtReport = [97, 224, 145, 99, 255, 223];
pub const CLASSIC_BTN_L: ExtReport = [97, 224, 241, 163, 223, 255];
pub const CLASSIC_BTN_R: ExtReport = [97, 224, 145, 124, 253, 255];
pub const CLASSIC_BTN_ZL: ExtReport = [97, 224, 145, 99, 255, 127];
pub const CLASSIC_BTN_ZR: ExtReport = [97, 224, 145, 99, 255, 251];
pub const CLASSIC_PAD_U: ExtReport = [97, 224, 145, 99, 255, 254];
pub const CLASSIC_PAD_D: ExtReport = [97, 224, 145, 99, 191, 255];
pub const CLASSIC_PAD_L: ExtReport = [97, 224, 145, 99, 255, 253];
pub const CLASSIC_PAD_R: ExtReport = [97, 224, 145, 99, 127, 255];
pub const CLASSIC_BTN_MINUS: ExtReport = [97, 224, 145, 99, 239, 255];
pub const CLASSIC_BTN_PLUS: ExtReport = [97, 224, 145, 99, 251, 255];
pub const CLASSIC_BTN_HOME: ExtReport = [97, 224, 145, 99, 247, 255];
pub const CLASSIC_LJOY_U: ExtReport = [97, 251, 145, 99, 255, 255];
pub const CLASSIC_LJOY_D: ExtReport = [97, 200, 145, 99, 255, 255];
pub const CLASSIC_LJOY_L: ExtReport = [72, 226, 145, 99, 255, 255];
pub const CLASSIC_LJOY_R: ExtReport = [121, 225, 145, 99, 255, 255];
pub const CLASSIC_RJOY_U: ExtReport = [161, 32, 29, 99, 255, 255];
pub const CLASSIC_RJOY_D: ExtReport = [161, 32, 3, 99, 255, 255];
pub const CLASSIC_RJOY_L: ExtReport = [33, 96, 144, 99, 255, 255];
pub const CLASSIC_RJOY_R: ExtReport = [225, 160, 16, 99, 255, 255];
pub const CLASSIC_LTRIG: ExtReport = [97, 224, 241, 195, 255, 255];
pub const CLASSIC_RTRIG: ExtReport = [97, 224, 145, 126, 255, 255];
pub const CLASSIC_LTRIG_W_BUTTON: ExtReport = [97, 224, 241, 195, 223, 255];
pub const CLASSIC_RTRIG_W_BUTTON: ExtReport = [97, 224, 145, 126, 253, 255];
pub const CLASSIC_LJOY_UR: ExtReport = [114, 243, 145, 99, 255, 255];
pub const CLASSIC_LJOY_DR: ExtReport = [114, 206, 145, 99, 255, 255];
pub const CLASSIC_LJOY_DL: ExtReport = [79, 205, 145, 99, 255, 255];
pub const CLASSIC_LJOY_UL: ExtReport = [79, 245, 145, 99, 255, 255];
pub const CLASSIC_RJOY_UR: ExtReport = [225, 32, 154, 99, 255, 255];
pub const CLASSIC_RJOY_DR: ExtReport = [225, 32, 134, 99, 255, 255];
pub const CLASSIC_RJOY_DL: ExtReport = [33, 224, 7, 99, 255, 255];
pub const CLASSIC_RJOY_UL: ExtReport = [33, 224, 25, 99, 255, 255];

// Wii Classic in High_Def mode (subset of all data, only really care about axis diffs)
pub const CLASSIC_HD_IDLE: ExtHdReport = [132, 127, 130, 136, 31, 26, 255, 255];
pub const CLASSIC_HD_LJOY_U: ExtHdReport = [134, 128, 238, 137, 31, 26, 255, 255];
pub const CLASSIC_HD_LJOY_D: ExtHdReport = [130, 128, 34, 138, 31, 26, 255, 255];
pub const CLASSIC_HD_LJOY_L: ExtHdReport = [36, 127, 135, 137, 31, 26, 255, 255];
pub const CLASSIC_HD_LJOY_R: ExtHdReport = [229, 127, 134, 138, 31, 26, 255, 255];
pub const CLASSIC_HD_RJOY_U: ExtHdReport = [132, 131, 130, 239, 31, 24, 255, 255];
pub const CLASSIC_HD_RJOY_D: ExtHdReport = [132, 130, 131, 30, 31, 24, 255, 255];
pub const CLASSIC_HD_RJOY_L: ExtHdReport = [133, 29, 130, 135, 31, 24, 255, 255];
pub const CLASSIC_HD_RJOY_R: ExtHdReport = [133, 226, 131, 132, 31, 24, 255, 255];
pub const CLASSIC_HD_LTRIG: ExtHdReport = [133, 128, 131, 137, 245, 22, 255, 255];
pub const CLASSIC_HD_RTRIG: ExtHdReport = [131, 128, 131, 137, 31, 230, 255, 255];
pub const CLASSIC_HD_BTN_X: ExtHdReport = [132, 128, 131, 137, 31, 26, 255, 247];

// wii classic pro joystick
pub const PRO_ID: ExtReport = [1, 0, 164, 32, 1, 1];
pub const PRO_HIRES_DEFAULT: u8 = 1;
pub const PRO_IDLE: ExtReport = [160, 31, 17, 0, 255, 255];
pub const PRO_BTN_B: ExtReport = [160, 31, 17, 0, 255, 191];
pub const PRO_BTN_A: ExtReport = [160, 31, 17, 0, 255, 239];
pub const PRO_BTN_X: ExtReport = [160, 31, 17, 0, 255, 247];
pub const PRO_BTN_Y: ExtReport = [160, 31, 17, 0, 255, 223];
pub const PRO_BTN_L: ExtReport = [159, 31, 113, 224, 223, 255];
pub const PRO_BTN_R: ExtReport = [160, 31, 17, 31, 253, 255];
pub const PRO_BTN_ZL: ExtReport = [160, 31, 17, 0, 255, 127];
pub const PRO_BTN_ZR: ExtReport = [160, 31, 17, 0, 255, 251];
pub const PRO_PAD_U: ExtReport = [160, 31, 17, 0, 255, 254];
pub const PRO_PAD_D: ExtReport = [160, 31, 17, 0, 191, 255];
pub const PRO_PAD_L: ExtReport = [160, 31, 17, 0, 255, 253];
pub const PRO_PAD_R: ExtReport = [160, 31, 17, 0, 127, 255];
pub const PRO_BTN_MINUS: ExtReport = [160, 31, 17, 0, 239, 255];
pub const PRO_BTN_PLUS: ExtReport = [160, 31, 17, 0, 251, 255];
pub const PRO_BTN_HOME: ExtReport = [160, 31, 17, 0, 247, 255];
pub const PRO_LJOY_U: ExtReport = [160, 57, 17, 0, 255, 255];
pub const PRO_LJOY_D: ExtReport = [160, 4, 17, 0, 255, 255];
pub const PRO_LJOY_L: ExtReport = [133, 30, 17, 0, 255, 255];
pub const PRO_LJOY_R: ExtReport = [185, 31, 17, 0, 255, 255];
pub const PRO_RJOY_U: ExtReport = [160, 31, 30, 0, 255, 255];
pub const PRO_RJOY_D: ExtReport = [160, 31, 4, 0, 255, 255];
pub const PRO_RJOY_L: ExtReport = [32, 95, 17, 0, 255, 255];
pub const PRO_RJOY_R: ExtReport = [224, 159, 145, 0, 255, 255];
pub const PRO_LJOY_UR: ExtReport = [180, 48, 17, 0, 255, 255];
pub const PRO_LJOY_DR: ExtReport = [179, 12, 17, 0, 255, 255];
pub const PRO_LJOY_DL: ExtReport = [139, 11, 17, 0, 255, 255];
pub const PRO_LJOY_UL: ExtReport = [139, 50, 17, 0, 255, 255];
pub const PRO_RJOY_UR: ExtReport = [224, 95, 27, 0, 255, 255];
pub const PRO_RJOY_DR: ExtReport = [224, 95, 7, 0, 255, 255];
pub const PRO_RJOY_DL: ExtReport = [32, 223, 7, 0, 255, 255];
pub const PRO_RJOY_UL: ExtReport = [32, 223, 27, 0, 255, 255];

// No analog triggers on pro controller
//pub const PRO_LTRIG: ExtReport = [];
//pub const PRO_RTRIG: ExtReport = [];

pub const PRO_HD_IDLE: ExtHdReport = [128, 129, 125, 139, 0, 0, 255, 255];
pub const PRO_HD_LJOY_U: ExtHdReport = [129, 129, 228, 139, 0, 0, 255, 255];
pub const PRO_HD_LJOY_D: ExtHdReport = [127, 129, 18, 139, 0, 0, 255, 255];
pub const PRO_HD_LJOY_L: ExtHdReport = [22, 129, 121, 139, 0, 0, 255, 255];
pub const PRO_HD_LJOY_R: ExtHdReport = [230, 129, 125, 139, 0, 0, 255, 255];
pub const PRO_HD_RJOY_U: ExtHdReport = [128, 130, 126, 245, 0, 0, 255, 255];
pub const PRO_HD_RJOY_D: ExtHdReport = [128, 130, 126, 33, 0, 0, 255, 255];
pub const PRO_HD_RJOY_L: ExtHdReport = [128, 23, 126, 142, 0, 0, 255, 255];
pub const PRO_HD_RJOY_R: ExtHdReport = [128, 235, 126, 139, 0, 0, 255, 255];
pub const PRO_HD_LJOY_UR: ExtHdReport = [207, 130, 197, 138, 0, 0, 255, 255];
pub const PRO_HD_LJOY_DR: ExtHdReport = [207, 130, 50, 138, 0, 0, 255, 255];
pub const PRO_HD_LJOY_DL: ExtHdReport = [45, 130, 43, 138, 0, 0, 255, 255];
pub const PRO_HD_LJOY_UL: ExtHdReport = [47, 130, 203, 138, 0, 0, 255, 255];
pub const PRO_HD_RJOY_UR: ExtHdReport = [128, 210, 125, 215, 0, 0, 255, 255];
pub const PRO_HD_RJOY_DR: ExtHdReport = [128, 210, 125, 62, 0, 0, 255, 255];
pub const PRO_HD_RJOY_DL: ExtHdReport = [128, 48, 125, 58, 0, 0, 255, 255];
pub const PRO_HD_RJOY_UL: ExtHdReport = [128, 50, 125, 220, 0, 0, 255, 255];

// No analog triggers on pro controller
//pub const PRO_HD_LTRIG: ExtReport = [];
//pub const PRO_HD_RTRIG: ExtReport = [];

// PDP "Link" gamecube clone controller
pub const PDP_LINK_ID: ExtReport = [1, 0, 164, 32, 1, 1];
pub const PDP_LINK_HIRES_DEFAULT: u8 = 1;
pub const PDP_LINK_IDLE: ExtReport = [160, 29, 15, 0, 255, 255];
pub const PDP_LINK_BTN_B: ExtReport = [160, 30, 15, 0, 255, 191];
pub const PDP_LINK_BTN_A: ExtReport = [160, 30, 15, 0, 255, 239];
pub const PDP_LINK_BTN_X: ExtReport = [160, 30, 15, 0, 255, 247];
pub const PDP_LINK_BTN_Y: ExtReport = [160, 30, 15, 0, 255, 223];
pub const PDP_LINK_BTN_L: ExtReport = [160, 30, 111, 224, 223, 255];
pub const PDP_LINK_BTN_R: ExtReport = [160, 30, 15, 31, 253, 255];
pub const PDP_LINK_BTN_ZL: ExtReport = [160, 30, 15, 0, 255, 127];
pub const PDP_LINK_BTN_ZR: ExtReport = [160, 30, 15, 0, 255, 251];
pub const PDP_LINK_PAD_U: ExtReport = [160, 30, 15, 0, 255, 254];
pub const PDP_LINK_PAD_D: ExtReport = [160, 30, 15, 0, 191, 255];
pub const PDP_LINK_PAD_L: ExtReport = [160, 30, 15, 0, 255, 253];
pub const PDP_LINK_PAD_R: ExtReport = [160, 30, 15, 0, 127, 255];
pub const PDP_LINK_BTN_MINUS: ExtReport = [160, 30, 15, 0, 239, 255];
pub const PDP_LINK_BTN_PLUS: ExtReport = [160, 30, 15, 0, 251, 255];
pub const PDP_LINK_BTN_HOME: ExtReport = [160, 30, 15, 0, 247, 255];
pub const PDP_LINK_LJOY_U: ExtReport = [159, 63, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_D: ExtReport = [159, 0, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_L: ExtReport = [128, 30, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_R: ExtReport = [189, 30, 15, 0, 255, 255];
pub const PDP_LINK_RJOY_U: ExtReport = [160, 30, 31, 0, 255, 255];
pub const PDP_LINK_RJOY_D: ExtReport = [160, 30, 0, 0, 255, 255];
pub const PDP_LINK_RJOY_L: ExtReport = [32, 30, 143, 0, 255, 255];
pub const PDP_LINK_RJOY_R: ExtReport = [224, 222, 143, 0, 255, 255];
// No analog triggers on PDP controller
// pub const PDP_LINK_LTRIG: ExtReport = [];
// pub const PDP_LINK_RTRIG: ExtReport = [];
pub const PDP_LINK_LJOY_UR: ExtReport = [180, 55, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_DR: ExtReport = [179, 6, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_DL: ExtReport = [131, 5, 15, 0, 255, 255];
pub const PDP_LINK_LJOY_UL: ExtReport = [131, 54, 15, 0, 255, 255];
pub const PDP_LINK_RJOY_UR: ExtReport = [223, 222, 27, 0, 255, 255];
pub const PDP_LINK_RJOY_DR: ExtReport = [223, 222, 2, 0, 255, 255];
pub const PDP_LINK_RJOY_DL: ExtReport = [31, 222, 2, 0, 255, 255];
pub const PDP_LINK_RJOY_UL: ExtReport = [31, 222, 27, 0, 255, 255];

pub const PDP_LINK_HD_IDLE: ExtHdReport = [126, 130, 120, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_U: ExtHdReport = [125, 130, 253, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_D: ExtHdReport = [124, 130, 0, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_L: ExtHdReport = [0, 130, 120, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_R: ExtHdReport = [245, 130, 123, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_U: ExtHdReport = [129, 132, 120, 253, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_D: ExtHdReport = [129, 132, 120, 0, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_L: ExtHdReport = [129, 10, 120, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_R: ExtHdReport = [129, 255, 120, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LTRIG: ExtHdReport = [129, 131, 120, 125, 0, 0, 255, 127];
pub const PDP_LINK_HD_RTRIG: ExtHdReport = [129, 131, 120, 125, 0, 0, 255, 251];
pub const PDP_LINK_HD_LJOY_UR: ExtHdReport = [209, 130, 222, 124, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_DR: ExtHdReport = [207, 131, 23, 125, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_DL: ExtHdReport = [18, 131, 20, 125, 0, 0, 255, 255];
pub const PDP_LINK_HD_LJOY_UL: ExtHdReport = [15, 131, 216, 125, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_UR: ExtHdReport = [127, 242, 120, 218, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_DR: ExtHdReport = [127, 242, 120, 19, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_DL: ExtHdReport = [127, 50, 120, 22, 0, 0, 255, 255];
pub const PDP_LINK_HD_RJOY_UL: ExtHdReport = [127, 50, 120, 219, 0, 0, 255, 255];
