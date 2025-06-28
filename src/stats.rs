//! Construct JSON string for easy handling of collected information.
//!
//! stats.rs
//! Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
//! 04-Jun-2025
//!

use crate::defs::*;
use crate::utils::*;
use log::{LevelFilter, debug, error, info, warn};
use std::sync::Arc;
use std::sync::Mutex;

pub fn get_json_obj(crypto_result: Arc<Mutex<CryptoResult>>) -> json::JsonValue {
    let c_r_p = crypto_result.lock().unwrap();
    let c_r: CryptoResult = c_r_p.clone();
    drop(c_r_p);

    let mut json_obj = json::JsonValue::new_object();

    json_obj[_J_TIME] = get_time_str().into();
    json_obj[_J_IP_ADDRESS] = get_ip().into();

    let (_uptime_secs, uptime_formatted, cpu_load, cpu_temp) = get_cpu_info();
    json_obj[_J_UPTIME] = uptime_formatted.into();
    json_obj[_J_LOAD] = cpu_load.into();
    json_obj[_J_CPU_TEMP] = cpu_temp.into();
    json_obj[_J_UPS_TIME] = 0.into();
    json_obj[_J_ON_BATTERY] = 0.into();
    json_obj[_J_BATTERY_PERCENT] = 0.into();
    json_obj[_J_NET_STATUS] = 0.into();
    json_obj[_J_TIME_REMAINING_OR_TO_FULL] = 0.into();
    json_obj[_J_PROCESS_NAME] = "firefox".into();
    json_obj[_J_PROCESS_STATUS] = 0.into();

    json_obj[_J_BTC_CMP] = c_r.btc_cmp.into();
    json_obj[_J_BTC_ATH] = c_r.btc_ath.into();
    json_obj[_J_BTC_CMP_ATH_DIFF] = c_r.btc_ath_cmp_diff.into();
    json_obj[_J_BTC_CMP_STR] = c_r.btc_cmp_str.into();
    json_obj[_J_BTC_ATH_STR] = c_r.btc_ath_str.into();
    json_obj[_J_BTC_CMP_ATH_DIFF_STR] = c_r.btc_ath_cmp_diff_str.into();

    debug!("{}(): {}", func_name!(), json_obj.dump());

    drop(crypto_result);
    json_obj
}

pub fn get_json_str(crypto_result: Arc<Mutex<CryptoResult>>) -> String {
    get_json_obj(crypto_result).dump()
}

// For JSON string output...
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
pub const _J_BTC_CMP: &str = "BTC_CMP";
pub const _J_BTC_ATH: &str = "BTC_ATH";
pub const _J_BTC_CMP_ATH_DIFF: &str = "BTC_CMP_ATH_DIFF";
pub const _J_BTC_CMP_STR: &str = "BTC_CMP_STR";
pub const _J_BTC_ATH_STR: &str = "BTC_ATH_STR";
pub const _J_BTC_CMP_ATH_DIFF_STR: &str = "BTC_CMP_ATH_DIFF_STR";
