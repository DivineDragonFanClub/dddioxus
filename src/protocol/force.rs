use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::FORCE_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForcesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForceInfo {
    pub id: i32,
    pub label: String,
    pub unit_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForcesResponse {
    pub forces: Vec<ForceInfo>,
}

impl Command for GetForcesRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 0);
    type Response = GetForcesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForceUnitsRequest {
    pub force_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatValue {
    pub label: String,
    pub index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitInfo {
    pub index: i32,
    pub name: String,
    pub level: i32,
    pub class: String,
    pub class_jid: String,
    pub stats: Vec<StatValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForceUnitsResponse {
    pub units: Vec<UnitInfo>,
}

impl Command for GetForceUnitsRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 1);
    type Response = GetForceUnitsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUnitStatRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub stat_index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUnitStatResponse {
    pub value: i32,
}

impl Command for SetUnitStatRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 2);
    type Response = SetUnitStatResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetClassesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassInfo {
    pub jid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetClassesResponse {
    pub classes: Vec<ClassInfo>,
}

impl Command for GetClassesRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 3);
    type Response = GetClassesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUnitClassRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub jid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUnitClassResponse {
    pub class: String,
    pub class_jid: String,
}

impl Command for SetUnitClassRequest {
    const ID: CommandId = CommandId::new(FORCE_NS, 4);
    type Response = SetUnitClassResponse;
}
