use std::error::Error;
use std::env;

mod modules;
mod utils;


fn main() -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let nodes_path = cwd.join("resources").join("nodes.json");
    let config = utils::config::parse_config(Some(nodes_path))?;

    let storage_path = cwd.join("resources").join("storage.json");
    let storage = modules::storages::json::JSONStorage::new(storage_path)?;

    let mut server = modules::server::Server::new(Box::new(storage), config);
    server.run()?;

    Ok(())
}
