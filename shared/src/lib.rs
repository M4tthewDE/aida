use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassLoadEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MethodEvent {
    Entry {
        timestamp: i64,
        name: String,
        class_name: String,
    },
    Exit {
        timestamp: i64,
        name: String,
        class_name: String,
    },
}

impl MethodEvent {
    pub fn timestamp(&self) -> i64 {
        match self {
            MethodEvent::Entry { timestamp, .. } => *timestamp,
            MethodEvent::Exit { timestamp, .. } => *timestamp,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MethodEvent::Entry { name, .. } => name,
            MethodEvent::Exit { name, .. } => name,
        }
    }

    pub fn class_name(&self) -> &str {
        match self {
            MethodEvent::Entry { class_name, .. } => class_name,
            MethodEvent::Exit { class_name, .. } => class_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    Unload,
    ClassLoad(ClassLoadEvent),
    MethodEvent(MethodEvent),
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
