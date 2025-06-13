/**
 * defs.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod defs {

    pub const HTTP_HOST: &str = "0.0.0.0";
    pub const HTTP_PORT: &str = "8080";

    pub type UBYTE = u8;
    pub type UWORD = u16;
    pub type UDOUBLE = u32;

    pub const LCD_HEIGHT: u16 = 128;
    pub const LCD_WIDTH: u16 = 128;

    pub const LCD_X_CORRECTION: u16 = 2;
    pub const LCD_Y_CORRECTION: u16 = 1;

    pub const LCD_ORIENTATION: LcdOrientation = LcdOrientation::Rotate0;

    pub const LCD_COLOUR_DEPTH: u16 = 2; // bytes

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

    #[derive(Debug)]
    pub enum LcdOrientation {
        Rotate0,
        Rotate90,
        Rotate180,
        Rotate270,
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
}
