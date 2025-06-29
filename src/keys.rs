//! Check display keys to change brightness or switch off or switch on the display
//!
//! keys.rs
//! Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
//! 01-Jun-2025
//!

use crate::defs::*;
use crate::pwm::*;
use crossbeam_channel::*;
use log::{LevelFilter, debug, error, info, warn};
use rppal::gpio::Gpio;
use signal_hook::consts::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Keys polling thread
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
}

/// USRSIG1 and USRSIG2 are used to switch on or switch off the display
/// using crontab. See LCD_crontab for details.
pub fn handle_usrsigs(
    s: Sender<BlMode>,
    sigusr_signals: &mut signal_hook::iterator::SignalsInfo,
    m: Arc<Mutex<bool>>,
) {
    'outer: loop {
        if let Some(signal) = sigusr_signals.pending().next() {
            match signal {
                SIGUSR1 => {
                    debug!("{}(): Recd SIGUSR1", func_name!());
                    s.send(BlMode::Off).unwrap();
                }
                SIGUSR2 => {
                    debug!("{}(): Recd SIGUSR2", func_name!());
                    s.send(BlMode::On).unwrap();
                }
                _ => {}
            }
        }
        let _exit = m.lock().unwrap();
        if *_exit {
            info!("Exiting {}()", func_name!());
            break 'outer;
        }
        drop(_exit);
    }
    drop(s);
}
