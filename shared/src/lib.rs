use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassLoadEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MethodEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    Unload,
    ClassLoad(ClassLoadEvent),
    MethodEntry(MethodEvent),
    MethodExit(MethodEvent),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub jar: String,
    pub class_loads: Vec<String>,
    pub methods: Vec<String>,
}

pub fn load_config(path: PathBuf) -> Config {
    let config_str = std::fs::read_to_string(path).unwrap();
    toml::from_str(&config_str).unwrap()
}
