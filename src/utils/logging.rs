use std::net::SocketAddr;
use chrono::Local;


pub fn log_message(ip: SocketAddr, message: String) {
    let now = Local::now().format("%d/%b/%Y:%H:%M:%S %z").to_string();
    println!("{} [{}] {}", ip, now, message);
}
