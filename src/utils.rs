/**
 * utils.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 05-Jun-2025
 *
 */
pub mod utils {
    use chrono::Local;
    use local_ip_address::local_ip;
    use systemstat::{Platform, System};

    pub const _J_TIME: &str = "TIME";
    pub const _J_IP_ADDRESS: &str = "IP_ADDRESS";
    pub const _J_UPTIME: &str = "UPTIME";
    pub const _J_LOAD: &str = "LOAD";
    pub const _J_CPU_TEMP: &str = "CPU_TEMP";
    pub const _J_CHARGE: &str = "CHARGE";
    pub const _J_UPS_TIME: &str = "UPS_TIME";
    pub const _J_ON_BATTERY: &str = "ON_BATTERY";
    pub const _J_BATTERY_PERCENT: &str = "BATTERY_PERCENT";
    pub const _J_NET_STATUS: &str = "NET_STATUS";
    pub const _J_TIME_REMAINING_OR_TO_FULL: &str = "TIME_REMAINING_OR_TO_FULL";
    pub const _J_PROCESS_NAME: &str = "PROCESS_NAME";
    pub const _J_PROCESS_STATUS: &str = "PROCESS_STATUS";

    pub fn get_ip() -> String {
        return String::from(local_ip().unwrap().to_string());
    }

    pub fn get_cpu_info() -> (String, String, String, String) {
        let sys = System::new();

        let uptime_seconds: String;
        let uptime_formated: String;
        let load_average: String;
        let cpu_temperature: String;

        match sys.uptime() {
            Ok(uptime) => {
                let secs: u64 = uptime.as_secs();
                let days: u64 = secs / (24 * 3600);
                let hours: u64 = (secs % (24 * 3600)) / (3600);
                uptime_seconds = String::from(format!("{:?}", secs));
                uptime_formated = String::from(format!("{days} d, {hours} h"));
            }
            Err(e) => {
                uptime_seconds = String::from(e.to_string());
                uptime_formated = String::from(e.to_string());
            }
        }
        match sys.load_average() {
            Ok(loadavg) => {
                load_average = String::from(format!("{:.2} %", loadavg.one));
            }
            Err(e) => {
                load_average = String::from(e.to_string());
            }
        }

        match sys.cpu_temp() {
            Ok(cpu_temp) => {
                cpu_temperature = String::from(format!("{:.2} C", cpu_temp)); // "deg" symbol => \u{00B0}
            }
            Err(e) => {
                cpu_temperature = String::from(e.to_string());
            }
        }

        return (
            uptime_seconds,
            uptime_formated,
            load_average,
            cpu_temperature,
        );
    }

    pub fn get_time_str() -> String {
        return format!("{}", Local::now().format("%H:%M %d-%b-%Y"));
    }
}
