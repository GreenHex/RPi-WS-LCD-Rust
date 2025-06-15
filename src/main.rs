#![allow(unused_imports, dead_code, unused_assignments, unused_variables)]
/**
 * dev.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 30-May-2025
 *
 */
use crossbeam_channel::{bounded, unbounded};
use log::{LevelFilter, debug, error, info, warn};
use rppal::gpio::{Gpio, Level};
use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};
use serde_json::json;
use serde_json::{Value, from_str};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use signal_hook::low_level::exit;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, u8};
use stdext::function_name;
use systemd_journal_logger::JournalLog;
mod keys;
use crate::keys::keys::*;
mod defs;
use crate::defs::defs::*;
mod gpio;
use crate::gpio::gpio::*;
mod pwm;
use crate::pwm::pwm::*;
mod spi;
use crate::spi::spi::*;
mod lcd;
use crate::lcd::lcd::*;
use CmdOrData::*;
mod usb;
use crate::usb::usb::usb_thd;
mod http;
use crate::http::http::http_server;
mod font12;
use crate::font12::font12::*;
mod font16;
use crate::font16::font16::*;
mod font20;
use crate::font20::font20::*;
mod font24;
use crate::font24::font24::*;
mod font50;
use crate::font50::font50::*;
mod font8;
use crate::font8::font8::*;
mod font48;
use crate::font48::font48::*;

use rand::Rng;
use terminate_thread::Thread;

fn main() -> Result<(), Box<dyn Error>> {
    let exe_name = std::env::current_exe()
        .expect("Can't get the exec path")
        .file_name()
        .expect("Can't get the exec name")
        .to_string_lossy()
        .into_owned();

    JournalLog::new()
        .unwrap()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(String::from("LCD"))
        //     systemd_journal_logger::current_exe_identifier()
        //         .expect("Error: systemd_journal_logger::current_exe_identifier()")
        //         .to_string(),
        // )
        .install()
        .unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("[{exe_name}] started");

    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_pwm = exit_flag.clone();
    let exit_flag_kchk = exit_flag.clone();
    let exit_usb_thd = exit_flag.clone();

    let (s1, r1) = unbounded::<BlMode>();
    let _s2 = s1.clone();

    let key_chk_thread: thread::JoinHandle<()> = thread::spawn(|| keys_check(s1, exit_flag_kchk));
    let pwm_thread: thread::JoinHandle<()> = thread::spawn(|| bl_pwm(r1, exit_flag_pwm));
    let usb_thread: thread::JoinHandle<()> = thread::spawn(|| usb_thd(exit_usb_thd));
    let http_server_thread = Thread::spawn(|| http_server());

    let mut l = Lcd::new(LCD_CS, LCD_DC, LCD_RST, LCD_BL)
        .with_orientation(LCD_ORIENTATION)
        .with_max_buffer_size(64);

    let _ = gpio_init(&mut l, _s2);

    while !term_now.load(Ordering::Relaxed) { // MAIN LOOP
    }

    let mut exit_flag_p = exit_flag.lock().unwrap();
    *exit_flag_p = true;
    drop(exit_flag_p);

    info!("[{exe_name}] Stopping threads...");

    l.lcd_clear(BLACK).unwrap();

    match usb_thread.join() {
        Ok(result) => {
            info!("usb_thd() thread ended");
        }
        Err(e) => {
            error!("Error stopping usb_thd() thread");
        }
    }

    match pwm_thread.join() {
        Ok(result) => {
            info!("bl_pwm() thread ended");
        }
        Err(e) => {
            error!("Error stopping bl_pwm() thread");
        }
    }

    match key_chk_thread.join() {
        Ok(result) => {
            info!("keys_check() thread ended");
        }
        Err(e) => {
            error!("Error stopping keys_check() thread");
        }
    }

    http_server_thread.terminate(); //does this even work?
    info!("http_server() thread (probably) ended");

    info!("[{exe_name}] exited");
    return Ok(());
}

fn gpio_init(l: &mut Lcd, s: crossbeam_channel::Sender<BlMode>) -> Result<(), Box<dyn Error>> {
    l.lcd_init();
    // l.lcd_set_window(0, 0, IMG_WIDTH, IMG_HEIGHT).unwrap();
    spi_write_ubyte2(&CmdOrData::Cmd(MEMORY_WRITE));
    l.img_clear(WHITE);

    // l.lcd_set_window(20, 20, LCD_WIDTH - 30, LCD_HEIGHT - 30)
    //    .unwrap();

    // l.lcd_set_window(0, 0, 64, 132).unwrap();
    l.img_draw_rect(0, 0, IMG_WIDTH, IMG_HEIGHT * 2, BLUE);
    l.img_draw_rect(10, 20, 50, 128, GREEN);
    l.img_draw_rect(60, 200, 20, 30, RED);
    // l.img_draw_pixel(260 / 2, 260 / 2, BLACK);
    l.img_draw_pixel(32, 132, WHITE);
    l.img_draw_pixel(64, 132, WHITE);
    l.img_draw_pixel(64, 29, WHITE);
    l.img_draw_pixel(64, 200, WHITE);

    // l.lcd_fill_rect(10, 10, 20, 30, RED).unwrap();
    // l.lcd_fill_rect(0, 0, 64, 128, GRED).unwrap();

    l.img_draw_char(20, 40, 'a', &FONT8, WHITE, BLACK);
    l.img_draw_char(32, 100, 'A', &FONT50, BLUE, RED);
    l.img_draw_string(10, 10, String::from("Hello"), &FONT16, RED, BLACK);

    l.img_draw_image(0, 0, IMG_WIDTH, IMG_HEIGHT);
    return Ok(());
}
