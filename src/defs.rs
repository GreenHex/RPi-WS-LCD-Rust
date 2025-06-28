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
pub const HTTP_CRYPTO_REQ_INTERVAL_SECS: u64 = 30 * 60; // 30 mins

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

/// This is for the 1.44" LCD screen
pub const LCD_WIDTH: usize = 128;
pub const LCD_HEIGHT: usize = 128;

pub const IMG_WIDTH: usize = LCD_WIDTH;
pub const IMG_HEIGHT: usize = LCD_HEIGHT;

pub const LCD_X_CORRECTION: u8 = 0;
pub const LCD_Y_CORRECTION: u8 = 0;

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

/// get current function name
#[macro_export]
macro_rules! func_name {
    () => {
        stdext::function_name!().rsplitn(2, "::").next().unwrap()
    };
}
pub(crate) use func_name;
