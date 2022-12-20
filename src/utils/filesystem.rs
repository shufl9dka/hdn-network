use std::path::Path;
use std::{fs, io};

/// Reads JSON-formatted file and returns object of specified type.
///
/// Uses [`serde`] module to deserialize the contents of the file.
///
/// [`serde`]: https://docs.rs/serde/0.9.0-rc2/serde/index.html
///
/// # Usage
/// ```
/// let strings: Vec<String> = read_json(&path)?;
/// let obj: Type = read_json(&path_to_obj)?;
/// ```
pub fn read_json<'a, T: serde::Deserialize<'a>>(file_path: &Path) -> io::Result<T> {
    let contents = fs::read_to_string(file_path)?;
    let result: T = serde_json::from_str(Box::leak(contents.into_boxed_str()))?;
    Ok(result)
}

/// Saves serializable object as JSON-formatted file
///
/// Uses [`serde`] module to serialize the object.
///
/// [`serde`]: https://docs.rs/serde/0.9.0-rc2/serde/index.html
///
/// # Usage
/// ```
/// save_json(&path, &object)?;
/// ```
pub fn save_json<T: serde::Serialize>(file_path: &Path, object: &T) -> io::Result<()> {
    let contents = serde_json::to_string(object)?;
    fs::write(file_path, contents)
}
