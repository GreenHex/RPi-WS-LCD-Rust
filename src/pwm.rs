/**
 * keys.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */

pub mod pwm {
    use log::{LevelFilter, debug, error, info, warn};
    use rppal::gpio::OutputPin;
    use rppal::gpio::{Gpio, Level, Pin};
    use std::error::Error;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use std::{thread, u8};

    use crate::defs::defs::*;
    use crate::gpio::gpio::*;
    use crate::http::http::stats::utils::get_cpu_info;

    const PERIOD_MS: u64 = 8;

    pub enum BlMode {
        Toggle = 1,
        Step = 2,
        Medium = 3,
    }

    pub fn bl_pwm(r: crossbeam_channel::Receiver<BlMode>, m: Arc<Mutex<bool>>) {
        let mut pulse = PERIOD_MS / 2; // starting value

        gpio_sleep_ms(1000); // wait for BL to switch on before rolling

        match gpio_get_output_pin(LCD_BL) {
            Ok(mut out_pin) => {
                loop {
                    out_pin
                        .set_pwm(
                            Duration::from_millis(PERIOD_MS),
                            Duration::from_millis(pulse),
                        )
                        .expect("Error: bl_pwm(): pin.set_pwm()");

                    debug!("{}(): pulse value: {pulse}", func_name!());

                    match r.recv() {
                        Ok(BlMode::Toggle) => {
                            if pulse > 0 {
                                pulse = 0;
                            } else {
                                pulse = PERIOD_MS / 2;
                            }
                            debug!("{}(): got: {:#?} {pulse}", func_name!(), 1);
                        }
                        Ok(BlMode::Step) => {
                            pulse = pulse + 2;

                            if pulse < 1 || pulse > PERIOD_MS {
                                pulse = 0;
                            }
                            debug!("{}(): got: {:#?} {pulse}", func_name!(), 2);
                        }
                        Ok(BlMode::Medium) => {
                            // from gpio_init()
                            pulse = PERIOD_MS / 4;
                        }
                        _ => {}
                    }
                    let _exit = m.lock().unwrap();
                    if *_exit {
                        info!("Exiting {}()", func_name!());
                        break;
                    }
                    drop(_exit);
                    thread::sleep(Duration::from_millis(300));
                }
            }
            Err(e) => {
                error!("{}(): {:?}", func_name!(), e);
            }
        }
        return;
    }
}
