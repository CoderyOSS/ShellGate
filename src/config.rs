use crate::types::{Config, GateError};

/// Load and parse TOML config file.
///
/// Spec: gate-server/spec.md > "Config file"
/// Tasks: 1.2
/// Pure function — file I/O only.
pub fn load_config(path: &str) -> Result<Config, GateError> {
    todo!("load_config: read TOML file at {path}, parse into Config struct, validate required fields")
}
