//! Write to or read from SPI interface to control LCD display.
//!
//! spi.rs
//! Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
//! 01-Jun-2025
//!

use crate::defs::*;
use crate::gpio::*;
use crate::lcd::lcd::*;
use log::{LevelFilter, error, info, warn};
use rppal::gpio::Level;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub const SPI_BUS: Bus = Bus::Spi0;
pub const SPI_SLAVE_SELECT: SlaveSelect = SlaveSelect::Ss0;
pub const SPI_FREQ_HZ: u32 = 100_000_000; // MAX: 250 MHz
pub const SPI_MODE: Mode = Mode::Mode0;

// macro_rules! get_spi {
//     () => {
//         Spi::new(SPI_BUS, SPI_SLAVE_SELECT, SPI_FREQ_HZ, SPI_MODE).expect("Error: Could not open SPI port")
//     };
// }

pub fn get_spi() -> rppal::spi::Spi {
    Spi::new(SPI_BUS, SPI_SLAVE_SELECT, SPI_FREQ_HZ, SPI_MODE)
        .expect("Error: Could not open SPI port")
}

pub fn spi_write_ubyte(data: UBYTE, is_cmd: bool) {
    match gpio_write2(LCD_DC, if is_cmd { Level::Low } else { Level::High }) {
        Ok(_) => match get_spi().write(&[data]) {
            Ok(_) => {}
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
            }
        },
        Err(e) => {
            error!("{}(): {:?}", func_name!(), e);
        }
    }
}

pub fn spi_write_data_uword(data: UWORD) {
    match gpio_write2(LCD_DC, Level::High) {
        Ok(_) => match get_spi().write(&[((data >> 8) & 0xFF) as UBYTE, (data & 0xFF) as UBYTE]) {
            Ok(_) => {}
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
            }
        },
        Err(e) => {
            error!("{}(): {:?}", func_name!(), e);
        }
    }
}

pub fn spi_write_ubyte2(cmd_or_data: &CmdOrData) {
    let mut val: UBYTE = 0;
    match cmd_or_data {
        CmdOrData::Cmd(cmd) => {
            match gpio_write2(LCD_DC, Level::Low) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}(): {:?}", func_name!(), e);
                }
            }
            val = *cmd;
        }
        CmdOrData::Data(data) => {
            match gpio_write2(LCD_DC, Level::High) {
                Ok(_) => {}
                Err(e) => {
                    error!("{}(): {:?}", func_name!(), e);
                }
            }
            val = *data;
        }
    }
    match get_spi().write(&[val]) {
        Ok(_) => {}
        Err(e) => {
            error!("{}(): {:?}", func_name!(), e);
        }
    }
}

pub fn spi_write_seq(sequence: &[CmdOrData]) {
    for item in sequence {
        spi_write_ubyte2(item);
    }
}

pub fn spi_write_data_array(data: &[UBYTE]) {
    match gpio_write2(LCD_DC, Level::High) {
        Ok(_) => match get_spi().write(data) {
            Ok(_) => {}
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
            }
        },
        Err(e) => {
            error!("{}(): {:?}", func_name!(), e);
        }
    }
}

pub fn spi_write_data_enable() {
    match gpio_write2(LCD_DC, Level::High) {
        Ok(_) => {}
        Err(e) => {
            error!("{}(): {:?}", func_name!(), e);
        }
    }
}
