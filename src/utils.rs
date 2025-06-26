/**
 * utils.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 05-Jun-2025
 *
 */
/// Collect information ("stats") from device and pass it on
/// to the requesting service.
use chrono::Local;
use local_ip_address::local_ip;
use log::{LevelFilter, debug, error, info, warn};
use systemstat::{Platform, System};

pub fn get_ip() -> String {
    local_ip().unwrap().to_string()
}

pub fn get_cpu_info() -> (String, String, String, String) {
    let sys = System::new();

    let (uptime_seconds, uptime_formated) = match sys.uptime() {
        Ok(uptime) => {
            let secs: u64 = uptime.as_secs();
            let days: u64 = secs / (24 * 3600);
            let hours: u64 = (secs % (24 * 3600)) / (3600);
            (format!("{:?}", secs), format!("{days} d, {hours} h"))
        }
        Err(e) => (e.to_string(), e.to_string()),
    };

    let load_average: String = match sys.load_average() {
        Ok(loadavg) => {
            format!("{:.2} %", loadavg.one)
        }
        Err(e) => e.to_string(),
    };

    let cpu_temperature: String = match sys.cpu_temp() {
        Ok(cpu_temp) => {
            format!("{:.2}\"C", cpu_temp) // "deg" symbol => \u{00B0}
        }
        Err(e) => e.to_string(),
    };

    (
        uptime_seconds,
        uptime_formated,
        load_average,
        cpu_temperature,
    )
}

pub fn get_time_str() -> String {
    format!("{}", Local::now().format("%H:%M %d-%b-%Y"))
}
