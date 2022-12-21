use std::error::Error;
use std::collections::HashMap;

use crate::modules::storages::basic::{Storage, StorageItem};

/// Storage type, which stores everything in RAM.
///
/// This type of storage DOESN'T save the changes forever, the storage will be empty after restart.
/// If you need to save your data forever, use [`JSONStorage`].
///
/// [`JSONStorage`]: #JSONStorage
pub struct MemoryStorage {
    storage_cache: HashMap<String, StorageItem>
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage {
        MemoryStorage {
            storage_cache: HashMap::new()
        }
    }
}

impl Storage for MemoryStorage {
    fn get_size(&self) -> usize {
        self.storage_cache.len()
    }

    fn get_storage_item(&self, key: &String) -> Result<Option<&StorageItem>, Box<dyn Error>> {
        Ok(self.storage_cache.get(key))
    }

    fn save_storage_item(&mut self, key: String, item: StorageItem) -> Result<(), Box<dyn Error>> {
        self.storage_cache.insert(key, item);
        Ok(())
    }

    fn get_entire_storage(&self) -> HashMap<String, StorageItem> {
        self.storage_cache.clone()
    }

    fn update_storage(&mut self, map: HashMap<String, StorageItem>) -> Result<(), Box<dyn Error>> {
        self.storage_cache = map;
        Ok(())
    }
}
