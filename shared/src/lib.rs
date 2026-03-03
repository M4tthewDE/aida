use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{class::ClassIdentifier, descriptor::MethodDescriptor};

pub mod class;
pub mod descriptor;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassLoadEvent {
    pub timestamp: i64,
    pub class_identifier: ClassIdentifier,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MethodEventType {
    Entry,
    Exit,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MethodEvent {
    pub timestamp: i64,
    pub name: String,
    pub class_identifier: ClassIdentifier,
    pub descriptor: MethodDescriptor,
    pub method_event_type: MethodEventType,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    Unload,
    ClassLoad(ClassLoadEvent),
    MethodEvent(MethodEvent),
}

#[derive(Deserialize, Debug)]
pub struct MethodConfig {
    pub name: String,
    pub class: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub jar: String,
    pub class_loads: Vec<String>,
    pub methods: Vec<MethodConfig>,
}

impl Config {
    pub fn includes_method(&self, name: &str, class: &str) -> bool {
        for method in &self.methods {
            if method.name == name && method.class == class {
                return true;
            }
        }

        false
    }
}

pub fn load_config(path: PathBuf) -> Config {
    let config_str = std::fs::read_to_string(path).unwrap();
    toml::from_str(&config_str).unwrap()
}
