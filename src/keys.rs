/**
 * keys.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod keys {
    use crossbeam_channel::*;
    use log::{LevelFilter, debug, error, info, warn};
    use rppal::gpio::{Gpio, Level};
    use signal_hook::consts::TERM_SIGNALS;
    use signal_hook::consts::*;
    use signal_hook::flag;
    use signal_hook::iterator::Signals;
    use signal_hook::low_level::exit;
    use signal_hook::*;
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

    pub fn handle_usrsigs<'wait>(
        s: Sender<BlMode>,
        sigusr_signals: &'wait mut signal_hook::iterator::SignalsInfo,
        m: Arc<Mutex<bool>>,
    ) {
        'outer: loop {
            'inner: for signal in sigusr_signals.pending() {
                match signal {
                    SIGUSR1 => {
                        debug!("{}(): Recd SIGUSR1", func_name!());
                        s.send(BlMode::Off).unwrap();
                        break 'inner;
                    }
                    SIGUSR2 => {
                        debug!("{}(): Recd SIGUSR2", func_name!());
                        s.send(BlMode::On).unwrap();
                        break 'inner;
                    }
                    _ => {
                        break 'inner;
                    }
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
}
