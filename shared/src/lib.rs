use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassLoadEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MethodEntryEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    Unload,
    ClassLoad(ClassLoadEvent),
    MethodEntry(MethodEntryEvent),
}
