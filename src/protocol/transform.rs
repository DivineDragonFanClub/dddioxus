use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::ENGAGE_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTransformRequest {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTransformResponse {
    pub path: String,
    pub local_position: Vec3,
    pub local_rotation: Vec3,
    pub local_scale: Vec3,
}

impl Command for GetTransformRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 2);
    type Response = GetTransformResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTransformRequest {
    pub path: String,
    pub local_position: Option<Vec3>,
    pub local_rotation: Option<Vec3>,
    pub local_scale: Option<Vec3>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTransformResponse {
    pub path: String,
    pub local_position: Vec3,
    pub local_rotation: Vec3,
    pub local_scale: Vec3,
}

impl Command for SetTransformRequest {
    const ID: CommandId = CommandId::new(ENGAGE_NS, 3);
    type Response = SetTransformResponse;
}
