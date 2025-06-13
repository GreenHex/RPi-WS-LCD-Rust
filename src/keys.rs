/**
 * keys.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod keys {
    use log::{LevelFilter, error, info, warn};
    use rppal::gpio::{Gpio, Level};
    use std::error::Error;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::{thread, u8};

    use crate::defs::defs::*;
    use crate::pwm::pwm::*;

    pub fn keys_check(s: crossbeam_channel::Sender<BlMode>, m: Arc<Mutex<bool>>) {
        let pin1 = Gpio::new().unwrap().get(KEY1).unwrap().into_input_pullup();
        let pin2 = Gpio::new().unwrap().get(KEY2).unwrap().into_input_pullup();

        loop {
            if pin1.is_low() {
                s.send(BlMode::Toggle).unwrap();
            } else if pin2.is_low() {
                s.send(BlMode::Step).unwrap();
            }
            let _exit = m.lock().unwrap();
            if *_exit {
                info!("Exiting {}()", func_name!());
                break;
            }
            drop(_exit);
            thread::sleep(Duration::from_millis(500));
        }
        return;
    }
}
