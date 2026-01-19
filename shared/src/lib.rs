use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum AgentMessage {
    VmInit,
    Load,
    Unload,
    ClassLoad(String),
}
