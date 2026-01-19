use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ClassLoadEvent {
    pub timestamp: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    VmInit,
    Load,
    Unload,
    ClassLoad(ClassLoadEvent),
}
