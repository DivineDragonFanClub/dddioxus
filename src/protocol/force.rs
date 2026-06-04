use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, FORCE_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForceInfo {
    pub id: i32,
    pub label: String,
    pub unit_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForcesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForcesResponse {
    pub forces: Vec<ForceInfo>,
}

impl Command for GetForcesRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 0);
    type Response = GetForcesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitSummary {
    pub index: i32,
    pub name: String,
    pub level: i32,
    pub class_jid: String,
    pub acted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitsRequest {
    pub force_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitsResponse {
    pub units: Vec<UnitSummary>,
}

impl Command for GetUnitsRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 1);
    type Response = GetUnitsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveUnitRequest {
    pub from_force_id: i32,
    pub unit_index: i32,
    pub to_force_id: i32,
}

impl Command for MoveUnitRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 2);
    type Response = OkResponse;
}
