/**
 * gpio.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod gpio {
    use log::{LevelFilter, debug, error, info, warn};
    use rppal::gpio::{Gpio, InputPin, IoPin, Level, OutputPin, Pin};
    use serialport::ErrorKind;
    use std::error::Error;
    use std::time::Duration;
    use std::{thread, u8};

    use crate::defs::defs::*;

    pub fn gpio_write(gpio_num: UBYTE, level: Level) -> Result<(), Box<dyn Error>> {
        let mut pin = Gpio::new()?.get(gpio_num)?.into_output();
        pin.write(level);
        drop(pin);

        return Ok(());
    }

    pub fn gpio_write2(gpio_num: UBYTE, level: Level) -> Result<(), Box<dyn Error>> {
        match Gpio::new() {
            Ok(gpio) => match gpio.get(gpio_num) {
                Ok(pin) => {
                    pin.into_output().write(level);
                }
                Err(e) => {
                    error!("{}(): {:?}", func_name!(), e);
                }
            },
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
            }
        }

        return Ok(());
    }

    pub fn gpio_read(gpio_num: UBYTE) -> Result<Level, Box<dyn Error>> {
        let pin = Gpio::new()?.get(gpio_num)?.into_input_pullup();

        return Ok(pin.read());
    }

    pub fn gpio_read2(gpio_num: UBYTE) -> Result<Level, Box<dyn Error>> {
        let err;
        match Gpio::new() {
            Ok(gpio) => match gpio.get(gpio_num) {
                Ok(pin) => {
                    return Ok(pin.into_input_pullup().read());
                }
                Err(e) => {
                    error!("{}(): {:?}", func_name!(), e);
                    err = e;
                }
            },
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
                err = e;
            }
        }
        return Err(err.into());
    }

    // this moves the pin out, there is no way of getting it back...
    pub fn gpio_get_output_pin(gpio_num: UBYTE) -> Result<OutputPin, Box<dyn Error>> {
        let err;
        match Gpio::new() {
            Ok(gpio) => match gpio.get(gpio_num) {
                Ok(pin) => {
                    return Ok(pin.into_output());
                }
                Err(e) => {
                    error!("{}(): {:?}", func_name!(), e);
                    err = e;
                }
            },
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
                err = e;
            }
        }

        return Err(err.into());
    }

    pub fn gpio_sleep_200_ms() {
        thread::sleep(Duration::from_millis(200));
    }

    pub fn gpio_sleep_ms(time_ms: u64) {
        thread::sleep(Duration::from_millis(time_ms));
    }
}
