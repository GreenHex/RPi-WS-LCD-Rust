/**
 * stats.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 04-Jun-2025
 *
 */

pub mod stats {
    use crate::defs::*;
    use json;
    use log::{LevelFilter, debug, error, info, warn};
    use std::error::Error;
    use std::sync::Arc;
    use std::sync::Mutex;

    include!("utils.rs");
    use utils::*;

    pub fn get_json_obj() -> json::JsonValue {
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

        debug!("{}(): {}", defs::func_name!(), json_obj.dump());

        return json_obj;
    }

    pub fn get_json_str() -> String {
        return get_json_obj().dump();
    }
}
