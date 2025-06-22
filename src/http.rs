/**
 * http.rs
 * Copyright (c) 2025 Vinodh Kumar Markapuram <GreenHex@gmail.com>
 * 05-Jun-2025
 *
 */

pub mod http {
    use crate::defs::defs::*;
    use crate::stats::stats::*;
    use log::{LevelFilter, debug, error, info, warn};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::{ascii::*, str::FromStr};
    use tiny_http::*;
    use tiny_http::{Response, Server};

    pub fn http_server(crypto_result: Arc<Mutex<CryptoResult>>) {
        let server_str = format!("{}:{}", HTTP_HOST, HTTP_PORT);

        let server = Server::http(&server_str).expect("Failed to start HTTP server");

        for request in server.incoming_requests() {
            debug!("{}(): got request", func_name!());
            debug!(
                "{}(): got request... method: {:?}, url: {:?}, headers: {:?}",
                func_name!(),
                request.method(),
                request.url(),
                request.headers()
            );
            let header = tiny_http::Header {
                field: tiny_http::HeaderField::from_str("Content-type")
                    .expect("Failed to set header field"),
                value: ascii::AsciiString::from_ascii("application/json; charset=utf-8")
                    .expect("Failed to set header value"),
            };
            request
                .respond(
                    Response::from_string(get_json_obj(crypto_result.clone()).to_string())
                        .with_header(header),
                )
                .expect("Failed sending request");
        }
        drop(crypto_result);
    }
}
