/**
 * crypto.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 05-Jun-2025
 *
 */
pub mod crypto {
    use crate::defs::defs::CryptoResult;
    use crate::defs::defs::*;
    use chrono::Local;
    use libc::CR0;
    use local_ip_address::local_ip;
    use log::{LevelFilter, debug, error, info, warn};
    use numfmt::{Formatter, Precision};
    use rusty_money::crypto;
    use rusty_money::{Money, iso};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use std::{thread, u8};
    use systemstat::{Platform, System};

    impl CryptoResult {
        pub fn new(
            btc_cmp: u64,
            btc_ath: u64,
            btc_ath_cmp_diff: i64,
            btc_cmp_str: String,
            btc_ath_str: String,
            btc_ath_cmp_diff_str: String,
        ) -> Self {
            Self {
                btc_cmp: btc_cmp,
                btc_ath: btc_ath,
                btc_ath_cmp_diff: btc_ath_cmp_diff,
                btc_cmp_str: btc_cmp_str,
                btc_ath_str: btc_ath_str,
                btc_ath_cmp_diff_str: btc_ath_cmp_diff_str,
            }
        }

        pub fn new_empty() -> Self {
            Self {
                btc_cmp: 0,
                btc_ath: 0,
                btc_ath_cmp_diff: 0,
                btc_cmp_str: String::from("waiting..."),
                btc_ath_str: String::from("waiting..."),
                btc_ath_cmp_diff_str: String::from("waiting..."),
            }
        }

        pub fn update(
            &mut self,
            btc_cmp: u64,
            btc_ath: u64,
            btc_ath_cmp_diff: i64,
            btc_cmp_str: String,
            btc_ath_str: String,
            btc_ath_cmp_diff_str: String,
        ) {
            self.btc_cmp = btc_cmp;
            self.btc_ath = btc_ath;
            self.btc_ath_cmp_diff = btc_ath_cmp_diff;
            self.btc_cmp_str = btc_cmp_str;
            self.btc_ath_str = btc_ath_str;
            self.btc_ath_cmp_diff_str = btc_ath_cmp_diff_str;
        }

        pub fn copy(&self, other: &mut Self) {
            other.btc_cmp = self.btc_cmp;
            other.btc_ath = self.btc_ath;
            other.btc_ath_cmp_diff = self.btc_ath_cmp_diff;
            other.btc_cmp_str = self.btc_cmp_str.clone();
            other.btc_ath_str = self.btc_ath_str.clone();
            other.btc_ath_cmp_diff_str = self.btc_ath_cmp_diff_str.clone();
        }

        pub fn get(self) -> (u64, u64, i64, String, String, String) {
            (
                self.btc_cmp,
                self.btc_ath,
                self.btc_ath_cmp_diff,
                self.btc_cmp_str,
                self.btc_ath_str,
                self.btc_ath_cmp_diff_str,
            )
        }

        pub fn print(self) {
            info!(
                "{}(): {} {} {}",
                func_name!(),
                Money::from_str(self.btc_ath.to_string().as_str(), iso::USD).unwrap(),
                Money::from_str(self.btc_cmp.to_string().as_str(), iso::USD).unwrap(),
                Money::from_str(
                    (self.btc_cmp as i64 - self.btc_ath as i64)
                        .to_string()
                        .as_str(),
                    iso::USD
                )
                .unwrap()
            );
        }
    }

    pub async fn crypto_thd(
        s: crossbeam_channel::Sender<CryptoResult>,
        m: Arc<Mutex<bool>>,
        crypto_result: Arc<Mutex<CryptoResult>>,
    ) {
        let mut c_r: CryptoResult = CryptoResult::new_empty();

        'outer: loop {
            c_r = get_btc().await;

            if c_r.btc_cmp > 0 {
                s.send(c_r.clone()).unwrap();

                let mut c_r_p = crypto_result.lock().unwrap();
                *c_r_p = c_r;
                drop(c_r_p);
            }

            // NOT REQUIRED...
            let _exit = m.lock().unwrap();
            if *_exit {
                info!("Exiting {}()", func_name!());
                break 'outer;
            }
            drop(_exit);
            thread::sleep(Duration::from_secs(HTTP_CRYPTO_REQ_INTERVAL_SECS));
        }
        drop(s);
        drop(crypto_result);
    }

    pub async fn get_btc() -> CryptoResult {
        let mut btc_ath = 0;
        let mut btc_cmp = 0;

        btc_cmp = match reqwest::get(HTTP_BTC_CMP_URL).await {
            Ok(resp) => {
                btc_ath = match reqwest::get(HTTP_BTC_ATH_URL).await {
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

        debug!(
            "{}(): {} {} {}",
            func_name!(),
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
