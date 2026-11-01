use std::{fs, path::Path};
use crate::resolved::ResolvedModel;

pub fn to_json(model: &ResolvedModel) -> Result<String, serde_json::Error> { serde_json::to_string_pretty(model) }
pub fn from_json(json: &str) -> Result<ResolvedModel, serde_json::Error> { serde_json::from_str(json) }
pub fn save_json(path: impl AsRef<Path>, model: &ResolvedModel) -> Result<(), StorageError> {
    fs::write(path, to_json(model)?)?; Ok(())
}
pub fn load_json(path: impl AsRef<Path>) -> Result<ResolvedModel, StorageError> {
    Ok(from_json(&fs::read_to_string(path)?)?)
}
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)] Io(#[from] std::io::Error),
    #[error(transparent)] Json(#[from] serde_json::Error),
}
