use crate::types::{Config, GateError};

pub fn load_config(path: &str) -> Result<Config, GateError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
