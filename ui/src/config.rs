use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub jar: String,
}

pub fn load(path: PathBuf) -> Config {
    let config_str = std::fs::read_to_string(path).unwrap();
    toml::from_str(&config_str).unwrap()
}
