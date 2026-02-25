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
pub enum MethodEvent {
    Entry {
        timestamp: i64,
        name: String,
        class_identifier: ClassIdentifier,
        descriptor: MethodDescriptor,
    },
    Exit {
        timestamp: i64,
        name: String,
        class_identifier: ClassIdentifier,
        descriptor: MethodDescriptor,
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

    pub fn class(&self) -> &ClassIdentifier {
        match self {
            MethodEvent::Entry {
                class_identifier, ..
            } => class_identifier,
            MethodEvent::Exit {
                class_identifier, ..
            } => class_identifier,
        }
    }

    pub fn descriptor(&self) -> &MethodDescriptor {
        match self {
            MethodEvent::Entry { descriptor, .. } => descriptor,
            MethodEvent::Exit { descriptor, .. } => descriptor,
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
