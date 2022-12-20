use std::error::Error;
use std::path::PathBuf;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use clap::Parser;

use crate::utils::filesystem;
use crate::utils::argparser::ArgsParser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub listen_ip: String,
    pub listen_port: u16,
    pub student_name: String,
    pub nodes_addrs: Vec<SocketAddr>,
}

/// Reads and returns the global config as a Config object.
///
/// Calls [`read_json`] to read the config file.
///
/// [`read_json`]: #read_json
///
/// # Usage
/// ```
/// let config: Config = read_config_file(file_path);
/// ```
pub fn read_config_file(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let config: Config = filesystem::read_json(&path.as_path())?;
    Ok(config)
}

/// Parses command line arguments and returns the config object.
///
/// You can pass the path to the file with the list of nodes as an argument.
/// Calls [`read_json`] to read the file with the list of nodes.
///
/// [`read_json`]: #read_json
///
/// # Usage
/// ```
/// let config: Config = parse_config(nodes_path);
/// ```
pub fn parse_config(nodes_path: Option<PathBuf>) -> Result<Config, Box<dyn Error>> {
    let args = ArgsParser::parse();
    let nodes_ips = match nodes_path {
        Some(path) => {
            let strings: Vec<String> = filesystem::read_json(&path.as_path())?;
            let mut res = vec![];
            for item in strings {
                let addrs: SocketAddr = item.parse()?;
                res.push(addrs);
            }
            res
        },
        None => { vec![] }
    };

    Ok(Config {
        listen_ip: args.host,
        listen_port: args.port,
        student_name: args.student,
        nodes_addrs: nodes_ips
    })
}
