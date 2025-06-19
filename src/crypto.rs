/**
 * crypto.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 05-Jun-2025
 *
 */
pub mod crypto {
    use crate::defs::defs::*;
    use chrono::Local;
    use local_ip_address::local_ip;
    use log::{LevelFilter, debug, error, info, warn};
    use numfmt::{Formatter, Precision};
    use rusty_money::{Money, iso};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use std::{thread, u8};
    use systemstat::{Platform, System};

    pub async fn crypto_thd(s: crossbeam_channel::Sender<CryptoResult>, m: Arc<Mutex<bool>>) {
        let mut crypto_result: CryptoResult = CryptoResult {
            btc_cmp: 0,
            btc_ath: 0,
            btc_ath_cmp_diff: 0,
            btc_cmp_str: String::from("waiting..."),
            btc_ath_str: String::from("waiting..."),
            btc_ath_cmp_diff_str: String::from("waiting..."),
        };

        'outer: loop {
            crypto_result = get_btc().await;

            if crypto_result.btc_cmp > 0 {
                let a = get_btc().await;
                s.send(a).unwrap();
            }

            // NOT REQUIRED...
            let _exit = m.lock().unwrap();
            if *_exit {
                info!("Exiting {}()", func_name!());
                break 'outer;
            }
            drop(_exit);
            thread::sleep(Duration::from_secs(HTTP_REQ_INTERVAL_SECS));
        }
        drop(s);
    }

    pub async fn get_btc() -> CryptoResult {
        let mut btc_ath = 0;
        let mut btc_cmp = 0;

        btc_cmp = match reqwest::get("https://cryptoprices.cc/BTC").await {
            Ok(resp) => {
                btc_ath = match reqwest::get("https://cryptoprices.cc/BTC/ATH").await {
                    Ok(resp) => resp
                        .text()
                        .await
                        .unwrap()
                        .replace("\n", "")
                        .parse::<u64>()
                        .unwrap(),
                    Err(e) => {
                        error!("Error: {:?}", e);
                        0
                    }
                };
                resp.text()
                    .await
                    .unwrap()
                    .replace("\n", "")
                    .parse::<u64>()
                    .unwrap()
            }
            Err(e) => {
                error!("Error: {:?}", e);
                0
            }
        };

        info!(
            "{} {} {}",
            Money::from_str(btc_ath.to_string().as_str(), iso::USD).unwrap(),
            Money::from_str(btc_cmp.to_string().as_str(), iso::USD).unwrap(),
            Money::from_str(
                (btc_cmp as i64 - btc_ath as i64).to_string().as_str(),
                iso::USD
            )
            .unwrap()
        );

        let mut f = Formatter::new() // start with blank representation
            .separator(',')
            .unwrap()
            .prefix("$")
            .unwrap()
            .precision(Precision::Decimals(0));

        return CryptoResult {
            btc_cmp: btc_cmp as u64,
            btc_ath: btc_ath as u64,
            btc_ath_cmp_diff: (btc_cmp as i64 - btc_ath as i64) as i64,
            btc_cmp_str: f.fmt2(btc_cmp).to_string(),
            btc_ath_str: f.fmt2(btc_ath).to_string(),
            btc_ath_cmp_diff_str: f.fmt2(btc_cmp as i64 - btc_ath as i64).to_string(),
        };
    }
}
