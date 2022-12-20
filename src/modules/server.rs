use std::collections::HashMap;
use std::error::Error;
// use std::thread;
use std::string::String;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};

use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds_option;

use crate::modules::storages::basic::{Storage, StorageItem};
use crate::utils::config::Config;
use crate::utils::logging::log_message;

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    request_type: String,
    key: Option<String>,
    hash: Option<String>,
    timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SrvResponse {
    response_status: String,
    storage: Option<HashMap<String, StorageItem>>
}

/// HDN server which works on raw sockets. Receives and processes users requests.
///
/// Provide it a storage object, config and run.
///
/// # Usage
/// ```
/// let mut server: Server = Server::new(MemoryStorage::new(), config);
/// server.run();  // server instance should be mutable to run it
/// ```
pub struct Server {
    storage: Box<dyn Storage>,
    config: Config,
}

impl Server {
    pub fn new(storage: Box<dyn Storage>, mut config: Config) -> Server {
        let my_ip = format!("{}:{}", config.listen_ip.clone(), config.listen_port.clone());
        let my_local_ip = format!("127.0.0.1:{}", config.listen_port.clone());
        let my_localhost_ip = format!("localhost:{}", config.listen_port.clone());

        config.nodes_addrs.retain(|&address| {  // removing self IP's
            let it_ip = address.to_string();
            it_ip != my_ip && it_ip != my_local_ip && it_ip != my_localhost_ip
        });

        Server {
            storage,
            config
        }
    }

    fn try_update(&mut self) {
        for addrs in self.config.nodes_addrs.iter() {
            match TcpStream::connect(addrs) {
                Ok(mut stream) => {
                    if serde_json::to_writer(&stream, &json!({
                        "request_type": "srv_update"
                    })).is_err() {
                        continue;
                    }
                    let mut de = serde_json::Deserializer::from_reader(&stream);
                    match SrvResponse::deserialize(&mut de) {
                        Ok(resp) => {
                            match resp.storage {
                                Some(map) => {
                                    self.storage.update_storage(map);
                                },
                                None => {
                                    continue;
                                }
                            }
                        },
                        Err(_) => {
                            continue;
                        }
                    }
                },
                Err(_) => {
                    continue;
                }
            }
        }
    }

    /// Runs server instance. That's it.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let ip = format!("{}:{}", &self.config.listen_ip, &self.config.listen_port);
        let listener = TcpListener::bind(ip)?;

        self.try_update();

        for stream in listener.incoming() {
            let stream = stream?;

            /* thread::spawn(|| {
                if self.handle_connection(stream).is_err() {
                    println!("Handle crashed unexpectedly");
                }
            }); */
            self.handle_connection(stream).ok();
        }
        Ok(())
    }

    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(&mut stream, &json!({
            "student_name": self.config.student_name.clone(),
        }))?;
        log_message(stream.local_addr()?, format!(
            "Connection established. Storage size: {}.", self.storage.get_size()
        ));

        let bad_request_resp = json!({
            "response_status": "bad request"
        });
        let server_error_resp = json!({
            "response_status": "server internal error"
        });

        let mut de = serde_json::Deserializer::from_reader(&stream);
        loop {
            match Request::deserialize(&mut de) {
                Ok(request) => {
                    if match request.request_type.as_str() {
                        "store" => self.process_store(&stream, request),
                        "load" => self.process_load(&stream, request),
                        "srv_store" => self.process_srv_store(&stream, request),
                        "srv_update" => self.process_srv_update(&stream, request),

                        _ => Ok(serde_json::to_writer(&stream, &bad_request_resp)?)
                    }.is_err() {
                        log_message(stream.local_addr()?, format!(
                            "Error occurred while processing a request: {:?}",
                            self.storage.get_size()
                        ));
                        serde_json::to_writer(&stream, &server_error_resp)?;
                    }
                },
                Err(_) => {
                    log_message(stream.local_addr()?, format!(
                        "Got bad request, connection terminated."
                    ));
                    serde_json::to_writer(&stream, &bad_request_resp)?;
                    break;
                }
            }
        }
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    fn broadcast_store(&mut self, request: Request) {
        let ts = Utc::now().timestamp_millis();

        for addrs in self.config.nodes_addrs.iter() {
            match TcpStream::connect(addrs) {
                Ok(mut stream) => {
                    if serde_json::to_writer(&stream, &json!({
                        "request_type": "srv_store",
                        "key": request.key.clone(),
                        "hash": request.hash.clone(),
                        "timestamp": ts.clone(),
                    })).is_err() {
                        continue;
                    }
                },
                Err(_) => {
                    continue;
                }
            }
        }
    }

    fn process_store(&mut self, stream: &TcpStream, request: Request) -> Result<(), Box<dyn Error>> {
        match (request.key.clone(), request.hash.clone()) {
            (Some(key), Some(hash)) => {
                self.storage.save_storage_item(key.clone(), StorageItem {
                    hash: hash.clone(),
                    pushed: Utc::now().timestamp_millis()
                })?;

                log_message(stream.local_addr()?, format!(
                    "Received request to write new value \"{}\" by key \"{}\". Storage size: {}.",
                    key.clone(), hash.clone(), self.storage.get_size()
                ));
                serde_json::to_writer(stream, &json!({
                    "response_status": "success"
                }))?;
                self.broadcast_store(request);
            },
            _ => {
                log_message(stream.local_addr()?, format!(
                    "Got \"store\" request with bad arguments."
                ));
                serde_json::to_writer(stream, &json!({
                    "response_status": "bad arguments"
                }))?;
            }
        }
        Ok(())
    }

    fn process_load(&mut self, stream: &TcpStream, request: Request) -> Result<(), Box<dyn Error>> {
        match request.key {
            Some(key) => {
                log_message(stream.local_addr()?, format!(
                    "Received request to get value by key \"{}\". Storage size: {}.",
                    key.clone(), self.storage.get_size()
                ));
                match self.storage.get_storage_item(&key)? {
                    Some(x) => {
                        serde_json::to_writer(stream, &json!({
                            "response_status": "success",
                            "requested_key": key.clone(),
                            "requested_hash": String::from(&x.hash)
                        }))?;
                    },
                    None => {
                        serde_json::to_writer(stream, &json!({
                            "response_status": "key not found"
                        }))?;
                    }
                }
            },
            _ => {
                log_message(stream.local_addr()?, format!(
                    "Got \"load\" request with bad arguments."
                ));
                serde_json::to_writer(stream, &json!({
                    "response_status": "bad arguments"
                }))?;
            }
        }
        Ok(())
    }

    fn check_is_node(&self, addrs: SocketAddr) -> bool {
        for it_addrs in self.config.nodes_addrs.iter() {
            if it_addrs.ip() == addrs.ip() {
                return true;
            }
        }
        false
    }

    fn process_srv_store(&mut self, stream: &TcpStream, request: Request) -> Result<(), Box<dyn Error>> {
        if !self.check_is_node(stream.local_addr()?) {
            serde_json::to_writer(stream, &json!({
                "response_status": "not a node"
            }))?;
            return Ok(());
        }
        match (request.key, request.hash, request.timestamp) {
            (Some(key), Some(hash), Some(timestamp)) => {
                let can_update = match self.storage.get_storage_item(&key)? {
                    None => { true }
                    Some(item) => { item.pushed < timestamp }
                };

                if can_update {
                    self.storage.save_storage_item(key.clone(), StorageItem {
                        hash: hash.clone(),
                        pushed: timestamp
                    })?;

                    serde_json::to_writer(stream, &json!({
                        "response_status": "success"
                    }))?;
                } else {
                    serde_json::to_writer(stream, &json!({
                        "response_status": "irrelevant update"
                    }))?;
                }

                log_message(stream.local_addr()?, format!(
                    "Received request to write new value \"{}\" by key \"{}\". Storage size: {}.",
                    key.clone(), hash.clone(), self.storage.get_size()
                ));
            },
            _ => {
                log_message(stream.local_addr()?, format!(
                    "Got \"store\" request with bad arguments."
                ));
                serde_json::to_writer(stream, &json!({
                    "response_status": "bad arguments"
                }))?;
            }
        }
        Ok(())
    }

    fn process_srv_update(&mut self, stream: &TcpStream, request: Request) -> Result<(), Box<dyn Error>> {
        if !self.check_is_node(stream.local_addr()?) {
            serde_json::to_writer(stream, &json!({
                "response_status": "not a node"
            }))?;
            return Ok(());
        }
        log_message(stream.local_addr()?, format!(
            "Received request to get entire storage from another server. Storage size: {}.",
            self.storage.get_size()
        ));

        let resp = SrvResponse {
            response_status: String::from("success"),
            storage: Some(self.storage.get_entire_storage())
        };
        serde_json::to_writer(stream, &resp)?;
        Ok(())
    }
}