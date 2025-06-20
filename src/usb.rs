/**
 * usb.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 04-Jun-2025
 *
 */

pub mod usb {
    use crate::defs::defs::*;
    use crate::stats::stats::*;
    use ascii::AsAsciiStr;
    use log::{LevelFilter, debug, error, info, warn};
    use serialport::*;
    use serialport::{SerialPortType, available_ports};
    use std::error::Error;
    use std::fs::read;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    const _CMD_READY: &str = ":READY:";
    const _CMD_OK: &str = ":OK:";
    const _CMD_FINISH: &str = ":FINISH:";
    const _CMD_RECEIVED: &str = ":RECEIVED:";
    const _CMD_RESEND: &str = ":RESEND:";
    const _CMD_RESET: &str = ":RESET:";
    const _CMD_OFF: &str = ":OFF:";
    const _CMD_ON: &str = ":ON:";

    const SPI_READ_BUFFER_SIZE: usize = 100_000;

    pub fn usb_thd(m: Arc<Mutex<bool>>, crypto_result: Arc<Mutex<CryptoResult>>) {
        'outer: loop {
            match available_ports() {
                Ok(ports) => {
                    if let Ok(port_name) = get_port(ports) {
                        let builder = serialport::new(port_name.clone(), 115_200)
                            .stop_bits(StopBits::One)
                            .data_bits(DataBits::Eight)
                            .parity(Parity::None);

                        info!("{}(): port_name: {}", func_name!(), port_name);

                        let port = builder.open_native().unwrap_or_else(|e| {
                            error!(
                                "{}(): Failed to open \"{}\". Error: {:?}",
                                func_name!(),
                                port_name,
                                e
                            );
                            ::std::process::exit(1);
                        });

                        match send_usb(port.try_clone().unwrap(), _CMD_ON) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("{}(): {:?}", func_name!(), e);
                            }
                        };
                        match read_usb(port.try_clone().unwrap(), _CMD_OK) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("{}(): {:?}", func_name!(), e);
                            }
                        }

                        match send_usb(port.try_clone().unwrap(), _CMD_RESET) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("{}(): {:?}", func_name!(), e);
                            }
                        }
                        match read_usb(port.try_clone().unwrap(), _CMD_OK) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("{}(): {:?}", func_name!(), e);
                            }
                        }

                        'inner: while port.try_clone().unwrap().read_clear_to_send().unwrap() {
                            match send_usb(port.try_clone().unwrap(), _CMD_READY) {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };
                            match read_usb(port.try_clone().unwrap(), _CMD_OK) {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            }

                            if break_check(port.try_clone().unwrap(), m.clone()) {
                                break 'outer;
                            }

                            match send_usb(
                                port.try_clone().unwrap(),
                                get_json_str(crypto_result.clone()).as_str(),
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };

                            match read_usb(port.try_clone().unwrap(), "") {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };

                            if break_check(port.try_clone().unwrap(), m.clone()) {
                                break 'outer;
                            }

                            thread::sleep(Duration::from_millis(15000));

                            if break_check(port.try_clone().unwrap(), m.clone()) {
                                break 'outer;
                            }

                            match send_usb(port.try_clone().unwrap(), _CMD_FINISH) {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };
                            match read_usb(port.try_clone().unwrap(), _CMD_RECEIVED) {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };

                            if break_check(port.try_clone().unwrap(), m.clone()) {
                                break 'outer;
                            }

                            thread::sleep(Duration::from_secs(10));

                            match read_usb(port.try_clone().unwrap(), "") {
                                Ok(_) => {}
                                Err(e) => {
                                    break 'inner;
                                }
                            };

                            if break_check(port.try_clone().unwrap(), m.clone()) {
                                break 'outer;
                            }
                        }
                    }
                }
                _ => {}
            }
            let _exit = m.lock().unwrap();

            if *_exit {
                info!("Exiting {}()", func_name!());
                break;
            }
        }
    }

    fn break_check(port: Box<dyn SerialPort + 'static>, m: Arc<Mutex<bool>>) -> bool {
        let mut retval = false;
        let _exit = m.lock().unwrap();

        if *_exit {
            send_usb(port.try_clone().unwrap(), _CMD_OFF).unwrap();
            read_usb(port.try_clone().unwrap(), _CMD_OK).unwrap();

            info!("Exiting {}()", func_name!());
            retval = true;
        }
        drop(_exit);
        return retval;
    }

    fn read_usb(mut port: Box<dyn SerialPort + 'static>, str: &str) -> Result<usize> {
        let mut read_buff: Vec<u8> = vec![0; SPI_READ_BUFFER_SIZE];

        match port.read(read_buff.as_mut_slice()) {
            Ok(read_buff_len) => {
                debug!(
                    "{}(): {}",
                    func_name!(),
                    std::str::from_utf8(&read_buff).unwrap().to_string()
                );
                debug!("{}(): Read {} bytes", func_name!(), read_buff_len);
                return Ok(read_buff_len);
            }
            Err(e) => {
                warn!("{}(): {:?}", func_name!(), e);
                return Err(e.into());
            }
        }
    }

    fn send_usb(mut port: Box<dyn SerialPort>, str: &str) -> Result<usize> {
        match port.read_clear_to_send() {
            Ok(true) => match port.write(str.as_bytes()) {
                Ok(a) => {
                    debug!("{}(): Wrote {} bytes", func_name!(), a);
                    thread::sleep(Duration::from_millis(1000));
                    return Ok(a);
                }
                Err(e) => {
                    warn!("{}(): {:?}", func_name!(), e);
                    return Err(e.into());
                }
            },
            Ok(false) => {
                warn!("{}(): {:?}", func_name!(), "Something bad happened");
                return Err(serialport::Error {
                    kind: ErrorKind::Unknown,
                    description: "Something bad happened".to_string(),
                });
            }
            Err(e) => {
                warn!("{}(): {:?}", func_name!(), e);
                return Err(e.into());
            }
        }
    }

    fn get_port(ports: Vec<SerialPortInfo>) -> Result<String> {
        let found_port: Option<SerialPortInfo> = ports.into_iter().find(|info| {
            if let SerialPortInfo {
                port_type: SerialPortType::UsbPort(info),
                ..
            } = info
            {
                if info.vid == USB_DEV_VENDOR_ID && info.pid == USB_DEV_PRODUCT_ID {
                    if info.serial_number == Some(String::from(USB_DEV_SERIAL_NUM)) {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        });

        return if let Some(SerialPortInfo { port_name, .. }) = found_port {
            Ok(String::from(port_name))
        } else {
            Err(serialport::Error {
                kind: ErrorKind::NoDevice,
                description: "Device not plugged in".to_string(),
            })
        };
    }
}
