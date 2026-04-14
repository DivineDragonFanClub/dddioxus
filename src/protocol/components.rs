use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::ENGAGE_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetComponentsRequest {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub index: u32,
    pub type_name: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetComponentsResponse {
    pub path: String,
    pub components: Vec<ComponentInfo>,
}

impl Command for GetComponentsRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 5);
    type Response = GetComponentsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleComponentRequest {
    pub path: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleComponentResponse {
    pub path: String,
    pub index: u32,
    pub enabled: bool,
}

impl Command for ToggleComponentRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 6);
    type Response = ToggleComponentResponse;
}
