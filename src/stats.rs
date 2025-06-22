/**
 * stats.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 04-Jun-2025
 *
 */

pub mod stats {
    use crate::defs::defs::*;
    use crate::utils::utils::*;
    use json;
    use log::{LevelFilter, debug, error, info, warn};
    use numfmt::{Formatter, Precision};
    use rusty_money::crypto;
    use rusty_money::{Money, iso};
    use std::error::Error;
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
}
