#![allow(unused_imports, dead_code, unused_assignments, unused_variables)]
#![warn(missing_docs)]
//!
//! main.rs
//! Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
//! 30-May-2025
//!

mod crypto;
mod defs;
mod fonts;
mod gpio;
mod http;
mod keys;
mod lcd;
mod pwm;
mod spi;
mod stats;
mod usb;
mod utils;

use crate::crypto::*;
use crate::defs::*;
use crate::fonts::font8::*;
use crate::fonts::font12::*;
use crate::fonts::font16::*;
use crate::http::http_server;
use crate::keys::*;
use crate::lcd::lcd::*;
use crate::pwm::*;
use crate::usb::usb_thd;
use crate::utils::*;
use crossbeam_channel::unbounded;
use log::{LevelFilter, debug, error, info, warn};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::consts::*;
use signal_hook::flag;
use signal_hook::iterator::Signals;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use systemd_journal_logger::JournalLog;
use tokio::runtime::Builder;

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

    // Trap Ctrl-C, other exit signals
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let mut sigusr_signals = Signals::new([SIGUSR1, SIGUSR2])?;

    // Flag(s) to signal thread loops to exit
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
    let usrsig_thread: thread::JoinHandle<()> =
        thread::spawn(move || handle_usrsigs(_s2, &mut sigusr_signals, exit_usrsigs_thd));

    // These blocking threads require tokio::rt
    let rt = Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .enable_all()
        .worker_threads(2) // TWO threads
        .build()
        .unwrap();
    let http_server_thread = rt.spawn(async { http_server(crypto_result1) });
    let crypto_thread = rt.spawn(crypto_thd(c_s1, exit_crypto_thd, crypto_result));

    let mut l = Lcd::new(LCD_CS, LCD_DC, LCD_RST, LCD_BL)
        .with_orientation(LCD_ORIENTATION)
        .with_max_buffer_size(64);

    l.lcd_init();

    // Don't know where to put this
    let mut btc: String = String::from("waiting...");

    // MAIN LOOP
    while !term_now.load(Ordering::Relaxed) {
        lcd_display_stuff(&mut l, &r_s1, &mut btc);

        thread::sleep(Duration::from_secs(SCREEN_UPDATE_INTERVAL_SECS));
    }

    info!("[{exe_name}] Stopping threads...");

    // Signal exit to all threads
    let mut exit_flag_p = exit_flag.lock().unwrap();
    *exit_flag_p = true;
    drop(exit_flag_p);

    l.lcd_clear(BLACK).unwrap();

    // Exit blocking threads
    crypto_thread.abort_handle().abort();
    info!("crypto_thd() ended");
    http_server_thread.abort_handle().abort();
    info!("http_server() thread ended");

    rt.shutdown_background();
    info!("tokio rt shutdown");

    match usrsig_thread.join() {
        Ok(_) => {
            info!("handle_usrsigs() thread ended");
        }
        Err(e) => {
            error!("Error stopping handle_usrsigs() thread {:?}", e);
        }
    }

    match usb_thread.join() {
        Ok(_) => {
            info!("usb_thd() thread ended");
        }
        Err(e) => {
            error!("Error stopping usb_thd() thread {:?}", e);
        }
    }

    match pwm_thread.join() {
        Ok(_) => {
            info!("bl_pwm() thread ended");
        }
        Err(e) => {
            error!("Error stopping bl_pwm() thread {:?}", e);
        }
    }

    match key_chk_thread.join() {
        Ok(_) => {
            info!("keys_check() thread ended");
        }
        Err(e) => {
            error!("Error stopping keys_check() thread {:?}", e);
        }
    }

    info!("[{exe_name}] exited");

    Ok(())
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

    l.img_draw_string(&(4), &(42), "IP Address", &FONT8, BLUE2, BLACK);
    l.img_draw_rect2(0, 42 + 24, IMG_WIDTH, FONT12.height, BLACK);
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
    l.img_draw_string(&(4), &(102), "Uptime", &FONT8, BLUE2, BLACK);
    l.img_draw_rect2(0, 102 + 24, IMG_WIDTH, FONT12.height, BLACK);
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

    l.img_draw_string(&(4), &(162), "Load", &FONT8, BLUE2, BLACK);
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
        "CPU Temp",
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

    if let Ok(crypto_result) = r_s1.try_recv() {
        *btc = crypto_result.btc_cmp_str.clone();
        crypto_result.print();
    };

    l.img_draw_string(
        &((IMG_WIDTH - btc.len() * FONT16.width) / 2),
        &(220 + 4),
        btc,
        &FONT16,
        BLACK,
        ORANGE,
    );

    l.img_draw_image(0, 0, LCD_WIDTH, LCD_HEIGHT);
}
