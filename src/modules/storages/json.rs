use std::error::Error;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::modules::storages::basic::{Storage, StorageItem};
use crate::utils::filesystem::{read_json, save_json};

/// Storage type, which stores everything in a json-like file.
///
/// Calls [`read_json`] once to read the storage file on its creation and [`save_json`]
/// everytime it has to save changes.
///
/// [`read_json`]: #read_json
/// [`save_json`]: #save_json
pub struct JSONStorage {
    filepath: PathBuf,
    storage_cache: HashMap<String, StorageItem>
}

impl JSONStorage {
    pub fn new(path: PathBuf) -> std::io::Result<JSONStorage> {
        Ok(JSONStorage {
            filepath: path.clone(),
            storage_cache: read_json(&path.as_path())?,
        })
    }
}

impl Storage for JSONStorage {
    fn get_size(&self) -> usize {
        self.storage_cache.len()
    }

    fn get_storage_item(&self, key: &String) -> Result<Option<&StorageItem>, Box<dyn Error>> {
        Ok(self.storage_cache.get(key))
    }

    fn save_storage_item(&mut self, key: String, item: StorageItem) -> Result<(), Box<dyn Error>> {
        self.storage_cache.insert(key, item);
        save_json(self.filepath.as_path(), &self.storage_cache)?;
        Ok(())
    }

    fn get_entire_storage(&self) -> HashMap<String, StorageItem> {
        self.storage_cache.clone()
    }

    fn update_storage(&mut self, map: HashMap<String, StorageItem>) -> Result<(), Box<dyn Error>> {
        self.storage_cache = map;
        save_json(self.filepath.as_path(), &self.storage_cache)?;
        Ok(())
    }
}
