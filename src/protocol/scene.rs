use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::ENGAGE_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSceneNameRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub name: String,
    pub path: String,
    pub active: bool,
    pub children: Vec<SceneNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneInfo {
    pub name: String,
    pub is_active: bool,
    pub objects: Vec<SceneNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSceneNameResponse {
    pub scene_name: String,
    pub scenes: Vec<SceneInfo>,
}

impl Command for GetSceneNameRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 0);
    type Response = GetSceneNameResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleGameObjectRequest {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleGameObjectResponse {
    pub path: String,
    pub active: bool,
}

impl Command for ToggleGameObjectRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 1);
    type Response = ToggleGameObjectResponse;
}
