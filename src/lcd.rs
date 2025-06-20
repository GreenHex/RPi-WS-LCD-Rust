/**
 * lcd.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod lcd {
    use crate::defs::defs::*;
    use crate::gpio::gpio::*;
    use crate::spi::spi::*;
    use log::{LevelFilter, debug, error, info, warn};
    use rppal::gpio::{Gpio, Level, OutputPin};
    use std::iter::*;
    use std::time::Duration;
    use std::{thread, u8};

    const DELAY: u64 = 200;
    const IMG_ARR_SIZE: usize = IMG_WIDTH * IMG_HEIGHT * LCD_COLOUR_DEPTH;

    // trait SPI {
    //     fn get_spi() -> rppal::spi::Spi;
    //     fn spi_write_ubyte(data: UBYTE, is_cmd: bool);
    //     fn spi_write_data_uword(data: UWORD);
    //     fn spi_write_seq(data: &[CmdOrData]);
    //     fn spi_write_data_array(data: &[UBYTE]);
    // }
    pub struct Lcd {
        cs_pin: UBYTE,
        dc_pin: UBYTE,  // Data / Command - 0=WriteCommand, 1=WriteData
        rst_pin: UBYTE, // Reset
        bl_pin: UBYTE,  // Backlight PWM
        orientation: LcdOrientation,
        max_buffer_size: usize,
        image: [u8; IMG_ARR_SIZE],
    }

    impl Lcd {
        pub fn new(cs_pin: UBYTE, dc_pin: UBYTE, rst_pin: UBYTE, bl_pin: UBYTE) -> Self {
            Self {
                cs_pin,
                dc_pin,
                rst_pin,
                bl_pin,
                orientation: LcdOrientation::Rotate0,
                max_buffer_size: 32,
                image: [0x00; IMG_ARR_SIZE],
            }
        }

        pub fn with_orientation(mut self, orientation: LcdOrientation) -> Self {
            self.orientation = orientation;
            self
        }

        pub fn with_max_buffer_size(mut self, size: usize) -> Self {
            self.max_buffer_size = size;
            self
        }

        fn size(&self) -> (usize, usize) {
            match self.orientation {
                LcdOrientation::Rotate0 => (LCD_WIDTH, LCD_HEIGHT),
                LcdOrientation::Rotate90 => (LCD_HEIGHT, LCD_WIDTH),
                LcdOrientation::Rotate180 => (LCD_WIDTH, LCD_HEIGHT),
                LcdOrientation::Rotate270 => (LCD_HEIGHT, LCD_WIDTH),
            }
        }

        fn set_adjustment(&self) -> (u8, u8) {
            if self.memory_access_control_value() & 0x20 != 0x20 {
                (LCD_X_CORRECTION, LCD_Y_CORRECTION)
            } else {
                (LCD_Y_CORRECTION, LCD_X_CORRECTION)
            }
        }

        fn memory_access_control_value(&self) -> UBYTE {
            let orientation = match self.orientation {
                LcdOrientation::Rotate0 => 0b00000000,
                LcdOrientation::Rotate90 => 0b01100000,
                LcdOrientation::Rotate180 => 0b11000000,
                LcdOrientation::Rotate270 => 0b10100000,
            };
            orientation | 0b00001000
        }

        pub fn lcd_init(&mut self) {
            gpio_write2(self.bl_pin, Level::High).unwrap();
            gpio_sleep_200_ms();
            gpio_write2(self.cs_pin, Level::High).unwrap();
            gpio_sleep_200_ms();
            gpio_write2(self.cs_pin, Level::Low).unwrap();
            gpio_sleep_200_ms();
            gpio_write2(self.rst_pin, Level::High).unwrap();
            gpio_sleep_200_ms();
            gpio_write2(self.rst_pin, Level::Low).unwrap();
            gpio_sleep_200_ms();
            gpio_write2(self.rst_pin, Level::High).unwrap();
            gpio_sleep_200_ms();

            spi_write_seq(&LCD_INIT_SEQ);

            spi_write_ubyte2(&CmdOrData::Cmd(0xB4)); // Column inversion
            spi_write_ubyte2(&CmdOrData::Data(0x07));

            // spi_write_seq(&LCD_GAMMA_SEQ);

            spi_write_ubyte2(&CmdOrData::Cmd(MEMORY_ACCESS_CONTROL));
            spi_write_ubyte2(&CmdOrData::Data(self.memory_access_control_value()));

            self.lcd_set_window(0, 0, LCD_WIDTH, LCD_HEIGHT).unwrap();

            self.lcd_clear(BLACK).unwrap();

            // self.img_clear(BLACK);
            // self.img_draw_image(0, 0, IMG_WIDTH, IMG_HEIGHT * 2);
        }

        pub fn lcd_set_window(
            &mut self,
            x0: usize,
            y0: usize,
            x1: usize,
            y1: usize,
        ) -> Result<(), LcdError> {
            let c1 = x1.saturating_sub(1).max(x0);
            let p1 = y1.saturating_sub(1).max(y0);
            let (c0h, c0l) = u16_to_bytes(x0 as u16);
            let (c1h, c1l) = u16_to_bytes(c1 as u16);
            let (p0h, p0l) = u16_to_bytes(y0 as u16);
            let (p1h, p1l) = u16_to_bytes(p1 as u16);

            let (x_adj, y_adj) = self.set_adjustment();

            spi_write_ubyte2(&CmdOrData::Cmd(PARTIAL_MODE_OFF));

            spi_write_ubyte2(&CmdOrData::Cmd(COLUMN_ADDRESS_SET));
            spi_write_data_array(&[c0h, c0l + x_adj, c1h, c1l & 0xFF + x_adj]);

            spi_write_ubyte2(&CmdOrData::Cmd(ROW_ADDRESS_SET));
            spi_write_data_array(&[p0h, p0l + y_adj, p1h, p1l & 0xFF + y_adj]);

            spi_write_ubyte2(&CmdOrData::Cmd(MEMORY_WRITE));

            Ok(())
        }

        pub fn lcd_fill_rect(
            &mut self,
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            colour: UWORD,
        ) -> Result<(), LcdError> {
            self.lcd_set_window(x, y, x + w, y + h)?;

            let chunk = color_buffer::<32>(colour);
            for _ in 0..(w as u32 * h as u32).div_ceil(16) {
                spi_write_data_array(&chunk);
            }
            Ok(())
        }

        pub fn lcd_clear(&mut self, colour: UWORD) -> Result<(), LcdError> {
            let (w, h) = self.size();
            self.lcd_fill_rect(0, 0, w, h, colour)?;

            Ok(())
        }

        /* image functions */
        pub fn img_clear(&mut self, colour: UWORD) -> &Self {
            for i in (0..IMG_ARR_SIZE).step_by(2) {
                self.image[i] = ((colour >> 8) & 0xFF) as u8;
                self.image[i + 1] = (colour & 0xFF) as u8;
            }
            self
        }

        pub fn img_draw_rect(
            &mut self,
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            colour: UWORD,
        ) -> &Self {
            for j in (y..(y + h)).step_by(2) {
                for i in (x..(x + w)).step_by(1) {
                    self.image[j * IMG_WIDTH + (i * LCD_COLOUR_DEPTH)] =
                        ((colour >> 8) & 0xFF) as u8;
                    self.image[j * IMG_WIDTH + (i * LCD_COLOUR_DEPTH) + 1] = (colour & 0xFF) as u8;
                }
            }
            self
        }

        pub fn img_draw_rect2(
            &mut self,
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            colour: UWORD,
        ) -> &Self {
            for j in (y..(y + h)).step_by(2) {
                for i in (x..(x + w)).step_by(1) {
                    self.img_draw_pixel(i, j, colour);
                }
            }
            self
        }

        pub fn img_draw_pixel(&mut self, x: usize, y: usize, colour: UWORD) -> &Self {
            let j = y.div_ceil(2) * 2;
            // if j >= LCD_HEIGHT {
            //     return self;
            // }
            self.image[j * IMG_WIDTH + x * LCD_COLOUR_DEPTH] = ((colour >> 8) & 0xFF) as u8;
            self.image[j * IMG_WIDTH + x * LCD_COLOUR_DEPTH + 1] = (colour & 0xFF) as u8;

            self
        }

        // same as img_draw_pixel(), don't know WTH is happening
        pub fn img_draw_pixel_font(&mut self, x: usize, y: usize, colour: UWORD) -> &Self {
            if y % 2 == 0 {
                self.image[y * IMG_WIDTH + x * LCD_COLOUR_DEPTH] = ((colour >> 8) & 0xFF) as u8;
                self.image[y * IMG_WIDTH + x * LCD_COLOUR_DEPTH + 1] = (colour & 0xFF) as u8;
            }
            self
        }

        pub fn img_draw_char<const N: usize>(
            &mut self,
            x: usize,
            y: usize,
            c: char,
            font: &FontTable<N>,
            colour_fg: UWORD,
            colour_bg: UWORD,
        ) {
            if c as u8 == 0 {
                error!("{}(): Char is NULL, exiting", func_name!());
                return;
            }

            if x > LCD_WIDTH || y > LCD_HEIGHT * 2 {
                error!(
                    "{}(): x value [{}] or y value [{}] is out of bounds, exiting",
                    func_name!(),
                    x,
                    y
                );
                return;
            }

            let mut char_offset: usize = (c as usize - ' ' as usize)
                * font.height
                * (font.width / 8 + (if font.width % 8 != 0 { 1 } else { 0 }));

            for j in (0..font.height).step_by(1) {
                for i in 0..font.width {
                    let pos = 0x80 >> (i % 8);
                    if (font.table[char_offset] & pos) != 0 {
                        self.img_draw_pixel_font(x + i, y + j * 2, colour_fg);
                    } else {
                        self.img_draw_pixel_font(x + i, y + j * 2, colour_bg);
                    }
                    if i % 8 == 7 {
                        char_offset += 1
                    }
                }
                if font.width % 8 != 0 {
                    char_offset += 1
                }
            }
        }

        pub fn img_draw_string<const N: usize>(
            &mut self,
            x: &usize,
            y: &usize,
            str: &String,
            font: &FontTable<N>,
            colour_fg: UWORD,
            colour_bg: UWORD,
        ) {
            let mut iter = str.chars();
            let mut x_pos = *x;
            loop {
                let ch = iter.next();
                match ch {
                    Some(c) => {
                        self.img_draw_char(x_pos, *y, c, font, colour_fg, colour_bg);
                        x_pos += font.width;
                    }
                    None => {
                        break;
                    }
                }
            }
        }

        pub fn img_draw_image(
            &mut self,
            x_start: usize,
            y_start: usize,
            x_end: usize,
            y_end: usize,
        ) -> &Self {
            // self.lcd_set_window(x_start, y_start, x_end, y_end).unwrap();

            let mut chunks = self.image.chunks((x_end - x_start) * LCD_COLOUR_DEPTH);

            loop {
                match chunks.next() {
                    Some(x) => {
                        spi_write_data_array(x);
                    }
                    None => {
                        break;
                    }
                }
            }
            self
        }

        // print array for debugging
        pub fn img_print_data(&self) {
            let mut chunks = self.image.chunks((IMG_WIDTH * LCD_COLOUR_DEPTH) as usize);

            loop {
                match chunks.next() {
                    Some(c) => {
                        for i in 0..(IMG_WIDTH * LCD_COLOUR_DEPTH) {
                            print!("0x{:02X} ", c[i]);
                        }
                        println!("\n");
                    }
                    None => {
                        println!("DONE\n");
                        break;
                    }
                }
            }
        }
    }
    #[derive(Debug)]
    pub enum LcdError {
        PinError,
        SpiError,
    }

    // REF: https://github.com/maciekglowka/lcd-ili9341-spi/blob/main/src/commands.rs
    pub const ENTER_SLEEP_MODE: u8 = 0x10;
    pub const SLEEP_OUT: u8 = 0x11;
    pub const PARTIAL_MODE_ON: u8 = 0x12;
    pub const PARTIAL_MODE_OFF: u8 = 0x13;
    pub const DISPLAY_INVERSION_OFF: u8 = 0x20;
    pub const DISPLAY_INVERSION_ON: u8 = 0x21;
    pub const GAMMA_SET: u8 = 0x26;
    pub const DISPLAY_OFF: u8 = 0x28;
    pub const DISPLAY_ON: u8 = 0x29;
    pub const COLUMN_ADDRESS_SET: u8 = 0x2A;
    pub const ROW_ADDRESS_SET: u8 = 0x2B;
    pub const MEMORY_WRITE: u8 = 0x2C;
    pub const VERTICAL_SCROLLING_DEFINITION: u8 = 0x33;
    pub const MEMORY_ACCESS_CONTROL: u8 = 0x36;
    pub const VERTICAL_SCROLLING_START_ADDRESS: u8 = 0x37;
    pub const PIXEL_FORMAT_SET: u8 = 0x3A;
    pub const SET_TEAR_SCANLINE: u8 = 0x44;
    pub const FRAME_CONTROL_NORMAL_MODE: u8 = 0xB1;
    pub const DISPLAY_FUNCTION_CONTROL: u8 = 0xB6;
    pub const POWER_CONTROL_1: u8 = 0xC0;
    pub const POWER_CONTROL_2: u8 = 0xC1;
    pub const VCOM_CONTROL_1: u8 = 0xC5;
    pub const VCOM_CONTROL_2: u8 = 0xC7;
    pub const POWER_CONTROL_A: u8 = 0xCB;
    pub const POWER_CONTROL_B: u8 = 0xCF;
    pub const POSITIVE_GAMMA_CORRECTION: u8 = 0xE0;
    pub const NEGATIVE_GAMMA_CORRECTION: u8 = 0xE1;
    pub const DRIVER_TIMING_CONTROL_A: u8 = 0xE8;
    pub const DRIVER_TIMING_CONTROL_B: u8 = 0xEA;
    pub const POWER_ON_SEQ_CONTROL: u8 = 0xED;
    pub const ENABLE_3G: u8 = 0xF2;
    pub const PUMP_RATIO_CONTROL: u8 = 0xF7;

    pub enum CmdOrData {
        Cmd(UBYTE),
        Data(UBYTE),
    }
    use CmdOrData::*;

    pub const LCD_INIT_SEQ: [CmdOrData; 47] = [
        Cmd(SLEEP_OUT),
        Cmd(POWER_CONTROL_B),
        Data(0x00),
        Data(0xC1),
        Data(0x30),
        Cmd(POWER_ON_SEQ_CONTROL),
        Data(0x64),
        Data(0x03),
        Data(0x12),
        Data(0x81),
        Cmd(DRIVER_TIMING_CONTROL_A),
        Data(0x85),
        Data(0x00),
        Data(0x79),
        Cmd(POWER_CONTROL_A),
        Data(0x39),
        Data(0x2C),
        Data(0x00),
        Data(0x34),
        Data(0x02),
        Cmd(PUMP_RATIO_CONTROL),
        Data(0x20),
        Cmd(DRIVER_TIMING_CONTROL_B),
        Data(0x00),
        Data(0x00),
        Cmd(POWER_CONTROL_1),
        Data(0x1D),
        Cmd(POWER_CONTROL_2),
        Data(0x12),
        Cmd(VCOM_CONTROL_1),
        Data(0x33),
        Data(0x3F),
        Cmd(VCOM_CONTROL_2),
        Data(0x92),
        Cmd(PIXEL_FORMAT_SET),
        Data(0x05), // 0x03 or 0x05 or 0x06?
        Cmd(MEMORY_ACCESS_CONTROL),
        Data(0x08),
        Cmd(FRAME_CONTROL_NORMAL_MODE),
        Data(0x00),
        Data(0x12),
        Cmd(DISPLAY_FUNCTION_CONTROL),
        Data(0x0A),
        Data(0xA2),
        Cmd(SET_TEAR_SCANLINE),
        Data(0x00),
        Cmd(DISPLAY_ON),
    ];

    pub const LCD_GAMMA_SEQ: [CmdOrData; 36] = [
        Cmd(ENABLE_3G),
        Data(0x00),
        Cmd(GAMMA_SET),
        Data(0x01),
        Cmd(POSITIVE_GAMMA_CORRECTION),
        Data(0x0F),
        Data(0x22),
        Data(0x1C),
        Data(0x1B),
        Data(0x08),
        Data(0x0F),
        Data(0x48),
        Data(0xB8),
        Data(0x34),
        Data(0x05),
        Data(0x0C),
        Data(0x09),
        Data(0x0F),
        Data(0x07),
        Data(0x00),
        Cmd(NEGATIVE_GAMMA_CORRECTION),
        Data(0x00),
        Data(0x23),
        Data(0x24),
        Data(0x07),
        Data(0x10),
        Data(0x07),
        Data(0x38),
        Data(0x47),
        Data(0x4B),
        Data(0x0A),
        Data(0x13),
        Data(0x06),
        Data(0x30),
        Data(0x38),
        Data(0x0F),
    ];

    // alternate initializing sequence
    pub const LCD_INIT_SEQ2: [CmdOrData; 76] = [
        Cmd(0xB1),
        Data(0x01),
        Data(0x2C),
        Data(0x2D),
        //
        Cmd(0xB2),
        Data(0x01),
        Data(0x2C),
        Data(0x2D),
        //
        Cmd(0xB3),
        Data(0x01),
        Data(0x2C),
        Data(0x2D),
        Data(0x01),
        Data(0x2C),
        Data(0x2D),
        //
        Cmd(0xB4), // Column inversion
        Data(0x07),
        //
        Cmd(0xC0), // ST7735R Power Sequence
        Data(0xA2),
        Data(0x02),
        Data(0x84),
        //
        Cmd(0xC1),
        Data(0xC5),
        //
        Cmd(0xC2),
        Data(0x0A),
        Data(0x00),
        //
        Cmd(0xC3),
        Data(0x8A),
        Data(0x2A),
        //
        Cmd(0xC4),
        Data(0x8A),
        Data(0xEE),
        //
        Cmd(0xC5), // VCOM
        Data(0x0E),
        //
        Cmd(0xE0), // ST7735R Gamma Sequence
        Data(0x0F),
        Data(0x1A),
        Data(0x0F),
        Data(0x18),
        Data(0x2F),
        Data(0x28),
        Data(0x20),
        Data(0x22),
        Data(0x1F),
        Data(0x1B),
        Data(0x23),
        Data(0x37),
        Data(0x00),
        Data(0x07),
        Data(0x02),
        Data(0x10),
        //
        Cmd(0xE1),
        Data(0x0F),
        Data(0x1B),
        Data(0x0F),
        Data(0x17),
        Data(0x33),
        Data(0x2C),
        Data(0x29),
        Data(0x2e),
        Data(0x30),
        Data(0x30),
        Data(0x39),
        Data(0x3F),
        Data(0x00),
        Data(0x07),
        Data(0x03),
        Data(0x10),
        //
        Cmd(0xF0), // Enable test command
        Data(0x01),
        //
        Cmd(0xF6), // Disable ram power save mode
        Data(0x00),
        //
        Cmd(0x3A), // 65k mode
        Data(0x05),
        //
        Cmd(0x11),
        Cmd(0x29),
    ];
}
