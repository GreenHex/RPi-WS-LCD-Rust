//! All the defs, The first few entries cab be customized.
//!
//! defs.rs
//! Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
//! 01-Jun-2025
//!

pub const SCREEN_UPDATE_INTERVAL_SECS: u64 = 5;

/// HTTP server to show statistics on a remote device (Raspberry
/// Pi Zero W with Waveshare 1.3" 240x240 display)
/// See: <https://github.com/GreenHex/Pico-HTTP-Remote-Status-Display>
pub const HTTP_HOST: &str = "0.0.0.0";
pub const HTTP_PORT: &str = "8080";

/// Free crypto prices server
pub const HTTP_BTC_CMP_URL: &str = "https://cryptoprices.cc/BTC";
pub const HTTP_BTC_ATH_URL: &str = "https://cryptoprices.cc/BTC/ATH";
pub const HTTP_CRYPTO_REQ_INTERVAL_SECS: u64 = 60 * 30;

/// ID and serial number of USB device (Raspberry Pi Zero with
/// Waveshare 1.3" 240x240 display) to show statistics on the
/// display.
/// See: <https://github.com/GreenHex/Pico-USB-Remote-Status-Display>
pub const USB_DEV_VENDOR_ID: u16 = 0x2E8A;
pub const USB_DEV_PRODUCT_ID: u16 = 0x000A;
pub const USB_DEV_SERIAL_NUM: &str = "E6616407E361442F";

pub type UBYTE = u8;
pub type UWORD = u16;
pub type UDOUBLE = u32;

pub const LCD_WIDTH: usize = 128;
pub const LCD_HEIGHT: usize = 128;

pub const IMG_WIDTH: usize = LCD_WIDTH;
pub const IMG_HEIGHT: usize = LCD_HEIGHT;

pub const LCD_X_CORRECTION: u8 = 0;
pub const LCD_Y_CORRECTION: u8 = 0;

pub const LCD_ORIENTATION: LcdOrientation = LcdOrientation::Rotate90;

pub const LCD_COLOUR_DEPTH: usize = 2; // bytes

pub const LCD_CLK: UBYTE = 11;
pub const LCD_MOSI: UBYTE = 10;

pub const LCD_CS: UBYTE = 8;
pub const LCD_RST: UBYTE = 27;
pub const LCD_DC: UBYTE = 25;
pub const LCD_BL: UBYTE = 24;

pub const KEY_UP: UBYTE = 6;
pub const KEY_DOWN: UBYTE = 19;
pub const KEY_LEFT: UBYTE = 5;
pub const KEY_RIGHT: UBYTE = 26;
pub const KEY_PRESS: UBYTE = 13;
pub const KEY1: UBYTE = 21;
pub const KEY2: UBYTE = 20;
pub const KEY3: UBYTE = 16;

pub const WHITE: UWORD = 0xFFFF;
pub const BLACK: UWORD = 0x0000;
pub const BLUE: UWORD = 0x001F;
pub const BRED: UWORD = 0xF81F;
pub const GRED: UWORD = 0xFFE0;
pub const GBLUE: UWORD = 0x07FF;
pub const RED: UWORD = 0xF800;
pub const MAGENTA: UWORD = 0xF81F;
pub const GREEN: UWORD = 0x07E0;
pub const CYAN: UWORD = 0x7FFF;
pub const YELLOW: UWORD = 0xFFE0;
pub const BROWN: UWORD = 0xBC40;
pub const BRRED: UWORD = 0xFC07;
pub const GRAY: UWORD = 0x8430;
pub const TANGARINE: UWORD = 0xFCCC;
pub const ORANGE: UWORD = 0xFE00;
pub const RED2: UWORD = 0xF841;
pub const BLUE2: UWORD = 0x0E3F;

#[derive(Debug)]
pub enum LcdOrientation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

pub struct FontTable<const N: usize> {
    pub table: [u8; N],
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct CryptoResult {
    pub btc_cmp: u64,
    pub btc_ath: u64,
    pub btc_ath_cmp_diff: i64,
    pub btc_cmp_str: String,
    pub btc_ath_str: String,
    pub btc_ath_cmp_diff_str: String,
}

#[macro_export]
macro_rules! func_name {
    () => {
        stdext::function_name!().rsplitn(2, "::").next().unwrap()
    };
}
pub(crate) use func_name;

// REF: https://github.com/maciekglowka/lcd-ili9341-spi/blob/main/src/utils.rs
pub(crate) fn u16_to_bytes(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, (val & 0xff) as u8)
}

/// Combine RGB channels into 565 RGB format - as u16
pub fn rgb_to_u16(r: u8, g: u8, b: u8) -> u16 {
    let rb = r >> 3;
    let gb = g >> 2;
    let bb = b >> 3;
    (rb as u16) << 11 | (gb as u16) << 5 | bb as u16
}

/// Combine RGB channels into 565 RGB format - as a (u8, u8) tuple
pub fn rgb_to_u8(r: u8, g: u8, b: u8) -> (u8, u8) {
    u16_to_bytes(rgb_to_u16(r, g, b))
}

/// Create a single colored buffer of N/2 pixel length
pub fn color_buffer<const N: usize>(color: u16) -> [u8; N] {
    let (h, l) = u16_to_bytes(color);
    core::array::from_fn(|i| if i % 2 == 0 { h } else { l })
}

// For JSON string output...
pub const _J_TIME: &str = "TIME";
pub const _J_IP_ADDRESS: &str = "IP_ADDRESS";
pub const _J_UPTIME: &str = "UPTIME";
pub const _J_LOAD: &str = "LOAD";
pub const _J_CPU_TEMP: &str = "CPU_TEMP";
pub const _J_CHARGE: &str = "CHARGE";
pub const _J_UPS_TIME: &str = "UPS_TIME";
pub const _J_ON_BATTERY: &str = "ON_BATTERY";
pub const _J_BATTERY_PERCENT: &str = "BATTERY_PERCENT";
pub const _J_NET_STATUS: &str = "NET_STATUS";
pub const _J_TIME_REMAINING_OR_TO_FULL: &str = "TIME_REMAINING_OR_TO_FULL";
pub const _J_PROCESS_NAME: &str = "PROCESS_NAME";
pub const _J_PROCESS_STATUS: &str = "PROCESS_STATUS";
pub const _J_BTC_CMP: &str = "BTC_CMP";
pub const _J_BTC_ATH: &str = "BTC_ATH";
pub const _J_BTC_CMP_ATH_DIFF: &str = "BTC_CMP_ATH_DIFF";
pub const _J_BTC_CMP_STR: &str = "BTC_CMP_STR";
pub const _J_BTC_ATH_STR: &str = "BTC_ATH_STR";
pub const _J_BTC_CMP_ATH_DIFF_STR: &str = "BTC_CMP_ATH_DIFF_STR";
