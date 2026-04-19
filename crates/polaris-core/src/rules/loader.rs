//! Load a rule spec from bytes or a filesystem path.

use thiserror::Error;

use super::schema::RuleSpec;

#[derive(Debug, Error)]
pub enum SpecLoadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn load_spec(bytes: &[u8]) -> Result<RuleSpec, SpecLoadError> {
    Ok(serde_json::from_slice(bytes)?)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_spec_from_path(path: &std::path::Path) -> Result<RuleSpec, SpecLoadError> {
    let bytes = std::fs::read(path)?;
    load_spec(&bytes)
}
