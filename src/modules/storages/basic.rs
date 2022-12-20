use std::error::Error;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A type which represents an item of storage
///
/// It stores a hash and the date when the element was pushed to the network.
/// Normally, you should access the item by it's key on the storage HashMap.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageItem {
    pub hash: String,
    pub pushed: i64
}

/// Basic trait for storages. That's it.
pub trait Storage {
    fn get_size(&self) -> usize;
    fn get_storage_item(&self, key: &String) -> Result<Option<&StorageItem>, Box<dyn Error>>;
    fn save_storage_item(&mut self, key: String, item: StorageItem) -> Result<(), Box<dyn Error>>;
    fn get_entire_storage(&self) -> HashMap<String, StorageItem>;
    fn update_storage(&mut self, map: HashMap<String, StorageItem>) -> Result<(), Box<dyn Error>>;
}
