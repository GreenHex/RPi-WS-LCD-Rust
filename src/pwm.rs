/**
 * pwm.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 01-Jun-2025
 *
 */
use crate::defs::*;
use crate::gpio::*;
use log::{LevelFilter, debug, error, info, warn};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const PERIOD_MS: u64 = 8;

pub enum BlMode {
    Toggle = 1,
    Step = 2,
    Mid = 3,
    Off = 4,
    On = 5,
}

pub fn bl_pwm(r: crossbeam_channel::Receiver<BlMode>, m: Arc<Mutex<bool>>) {
    let mut pulse: u64 = PERIOD_MS / 2; // starting value
    let mut old_pulse_val: u64 = 0;

    gpio_sleep_ms(1000); // wait for BL to switch on before rolling

    match gpio_get_output_pin(LCD_BL) {
        Ok(mut out_pin) => {
            loop {
                if old_pulse_val != pulse {
                    out_pin
                        .set_pwm(
                            Duration::from_millis(PERIOD_MS),
                            Duration::from_millis(pulse),
                        )
                        .expect("Error: bl_pwm(): pin.set_pwm()");

                    debug!("{}(): pulse value: {pulse}", func_name!());
                    old_pulse_val = pulse;
                }
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
                        pulse += 2;

                        if !(1..=PERIOD_MS).contains(&pulse) {
                            pulse = 0;
                        }
                        debug!("{}(): got: {:#?} {pulse}", func_name!(), 2);
                    }
                    Ok(BlMode::Mid) => {
                        // from gpio_init()
                        pulse = PERIOD_MS / 4;
                    }
                    Ok(BlMode::Off) => {
                        // SIGUSR1
                        pulse = 0;
                    }
                    Ok(BlMode::On) => {
                        // SIGUSR2
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
}
