use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, SKILL_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillInfo {
    pub sid: String,
    pub name: String,
    pub source: String,
    pub removable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillsResponse {
    pub skills: Vec<SkillInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitSkillsRequest {
    pub force_id: i32,
    pub unit_index: i32,
}

impl Command for GetUnitSkillsRequest {
    const ID: CommandId = CommandId::new(SKILL_NS, 0);
    type Response = SkillsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPersonSkillsRequest {
    pub force_id: i32,
    pub unit_index: i32,
}

impl Command for GetPersonSkillsRequest {
    const ID: CommandId = CommandId::new(SKILL_NS, 1);
    type Response = SkillsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddSkillRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub sid: String,
    pub target: String,
}

impl Command for AddSkillRequest {
    const ID: CommandId = CommandId::new(SKILL_NS, 2);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveSkillRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub sid: String,
    pub source: String,
}

impl Command for RemoveSkillRequest {
    const ID: CommandId = CommandId::new(SKILL_NS, 3);
    type Response = OkResponse;
}
