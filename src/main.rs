#![allow(unused_imports, dead_code, unused_assignments, unused_variables)]
/**
 * main.rs
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
use signal_hook::consts::*;
use signal_hook::flag;
use signal_hook::iterator::Signals;
use signal_hook::low_level::exit;
use signal_hook::*;
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
mod fonts;
use crate::fonts::font8::font8::*;
use crate::fonts::font12::font12::*;
use crate::fonts::font16::font16::*;
use crate::fonts::font20::font20::*;
use crate::fonts::font24::font24::*;
use crate::fonts::font48::font48::*;
use crate::fonts::font50::font50::*;
use rand::Rng;
use terminate_thread::Thread;
mod utils;
use crate::utils::utils::*;
mod crypto;
use crypto::crypto::*;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::time::*;
mod stats;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_name = std::env::current_exe()
        .expect("Can't get the exec path")
        .file_name()
        .expect("Can't get the exec name")
        .to_string_lossy()
        .into_owned();

    JournalLog::new()
        .unwrap()
        .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
        .with_syslog_identifier(
            // String::from(exe_name.clone()))
            systemd_journal_logger::current_exe_identifier()
                .expect("Error: systemd_journal_logger::current_exe_identifier()")
                .to_string(),
        )
        .install()
        .unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("[{exe_name}] started");

    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let mut sigusr_signals = Signals::new(&[SIGUSR1, SIGUSR2])?;

    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_pwm = exit_flag.clone();
    let exit_flag_kchk = exit_flag.clone();
    let exit_usb_thd = exit_flag.clone();
    let exit_crypto_thd = exit_flag.clone();
    let exit_usrsigs_thd = exit_flag.clone();

    let (s1, r1) = unbounded::<BlMode>(); // keys_check(), bl_pwm()
    let _s2 = s1.clone(); // forward signals to bl_pwm()

    let (c_s1, r_s1) = unbounded::<CryptoResult>(); // crypto_thd()

    let crypto_result = Arc::new(Mutex::new(CryptoResult::new_empty())); // crypto_thd()
    let crypto_result1 = crypto_result.clone(); // http_server()
    let crypto_result2 = crypto_result.clone(); // usb_thd()

    let key_chk_thread: thread::JoinHandle<()> = thread::spawn(|| keys_check(s1, exit_flag_kchk));
    let pwm_thread: thread::JoinHandle<()> = thread::spawn(|| bl_pwm(r1, exit_flag_pwm));
    let usb_thread: thread::JoinHandle<()> =
        thread::spawn(|| usb_thd(exit_usb_thd, crypto_result2));
    // let http_server_thread = Thread::spawn(|| http_server(crypto_result1));
    let usrsig_thread: thread::JoinHandle<()> =
        thread::spawn(move || handle_usrsigs(_s2, &mut sigusr_signals, exit_usrsigs_thd));

    let rt = Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .enable_all()
        .worker_threads(3)
        .build()
        .unwrap();
    let http_server_thread = rt.spawn(async { http_server(crypto_result1) });
    let crypto_thread = rt.spawn(crypto_thd(c_s1, exit_crypto_thd, crypto_result));
    // let usrsig_thread = rt.spawn(handle_usrsigs(_s2, &mut sigusr_signals));

    let mut l = Lcd::new(LCD_CS, LCD_DC, LCD_RST, LCD_BL)
        .with_orientation(LCD_ORIENTATION)
        .with_max_buffer_size(64);

    let _ = lcd_setup(&mut l);
    let mut btc: String = String::from("waiting...");

    // MAIN LOOP
    while !term_now.load(Ordering::Relaxed) {
        lcd_display_stuff(&mut l, &r_s1, &mut btc);

        thread::sleep(Duration::from_secs(SCREEN_UPDATE_INTERVAL_SECS));
    }

    info!("[{exe_name}] Stopping threads...");

    let mut exit_flag_p = exit_flag.lock().unwrap();
    *exit_flag_p = true;
    drop(exit_flag_p);

    l.lcd_clear(BLACK).unwrap();

    crypto_thread.abort_handle().abort();
    info!("crypto_thd() ended");
    http_server_thread.abort_handle().abort();
    info!("http_server() thread ended");

    rt.shutdown_background();
    info!("tokio rt shutdown");

    match usrsig_thread.join() {
        Ok(result) => {
            info!("handle_usrsigs() thread ended");
        }
        Err(e) => {
            error!("Error stopping handle_usrsigs() thread");
        }
    }

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

    info!("[{exe_name}] exited");

    Ok(())
}

fn lcd_setup(l: &mut Lcd) -> Result<(), Box<dyn Error>> {
    l.lcd_init();
    spi_write_ubyte2(&CmdOrData::Cmd(MEMORY_WRITE));

    l.img_clear(BLACK);

    return Ok(());
}

fn lcd_display_stuff(
    l: &mut Lcd,
    r_s1: &crossbeam_channel::Receiver<CryptoResult>,
    btc: &mut String,
) {
    l.lcd_set_window(0, 0, IMG_WIDTH, IMG_HEIGHT).unwrap();

    l.img_draw_rect2(0, 0, IMG_WIDTH, 32, WHITE);
    l.img_draw_string(
        &((IMG_WIDTH - get_time_str().len() * FONT12.width) / 2),
        &(8),
        &(get_time_str()),
        &FONT12,
        BLACK,
        WHITE,
    );

    l.img_draw_string(
        &(4),
        &(42),
        &("IP Address".to_string()),
        &FONT8,
        BLUE2,
        BLACK,
    );
    l.img_draw_string(
        &((IMG_WIDTH - get_ip().len() * FONT12.width) - 4),
        &(42 + 24),
        &(get_ip()),
        &FONT12,
        WHITE,
        BLACK,
    );
    l.img_draw_rect2(0, 42 + 24 + 2 + 2 + FONT12.height * 2, IMG_WIDTH, 1, ORANGE);

    let (_, uptime, load, temp) = get_cpu_info();
    l.img_draw_string(&(4), &(102), &("Uptime".to_string()), &FONT8, BLUE2, BLACK);
    l.img_draw_string(
        &((IMG_WIDTH - uptime.len() * FONT12.width) - 4),
        &(102 + 24),
        &(uptime),
        &FONT12,
        WHITE,
        BLACK,
    );
    l.img_draw_rect2(
        0,
        102 + 24 + 2 + 2 + FONT12.height * 2,
        IMG_WIDTH,
        1,
        ORANGE,
    );

    l.img_draw_string(&(4), &(162), &("Load".to_string()), &FONT8, BLUE2, BLACK);
    l.img_draw_string(
        &((IMG_WIDTH / 2 - load.len() * FONT12.width) - 4),
        &(162 + 24),
        &(load),
        &FONT12,
        WHITE,
        BLACK,
    );

    l.img_draw_string(
        &(IMG_WIDTH / 2 + 6),
        &(162),
        &("Temp".to_string()),
        &FONT8,
        BLUE2,
        BLACK,
    );
    l.img_draw_string(
        &(IMG_WIDTH / 2 + (IMG_WIDTH / 2 - temp.len() * FONT12.width) - 4),
        &(162 + 24),
        &(temp),
        &FONT12,
        WHITE,
        BLACK,
    );

    l.img_draw_rect2(
        IMG_WIDTH / 2,
        102 + 24 + 2 + 2 + FONT12.height * 2,
        1,
        64,
        ORANGE,
    );

    l.img_draw_rect2(1, 218, IMG_WIDTH - 2, FONT16.height * 2 + 2 + 2 + 2, ORANGE);

    match r_s1.try_recv() {
        Ok(crypto_result) => {
            *btc = crypto_result.btc_cmp_str.clone();
            crypto_result.print();
        }
        _ => {}
    };

    l.img_draw_string(
        &((IMG_WIDTH - btc.len() * FONT16.width) / 2),
        &(220 + 4),
        &btc,
        &FONT16,
        BLACK,
        ORANGE,
    );

    l.img_draw_image(0, 0, LCD_WIDTH, LCD_HEIGHT);

    return;
}
