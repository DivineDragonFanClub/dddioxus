use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::UNIT_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatValue {
    pub label: String,
    pub index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetStatsRequest {
    pub force_id: i32,
    pub unit_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetStatsResponse {
    pub stats: Vec<StatValue>,
}

impl Command for GetStatsRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 0);
    type Response = GetStatsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetStatRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub stat_index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetStatResponse {
    pub value: i32,
}

impl Command for SetStatRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 1);
    type Response = SetStatResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetClassRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub jid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetClassResponse {
    pub class_jid: String,
}

impl Command for SetClassRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 2);
    type Response = SetClassResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetActedRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub acted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetActedResponse {
    pub acted: bool,
}

impl Command for SetActedRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 3);
    type Response = SetActedResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    pub level: i32,
    pub internal_level: i32,
    pub total_level: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLevelRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub level: i32,
    pub grow_stats: bool,
}

impl Command for SetLevelRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 4);
    type Response = LevelInfo;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetInternalLevelRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub internal_level: i32,
    pub grow_stats: bool,
}

impl Command for SetInternalLevelRequest {
    const ID: CommandId = CommandId::new(UNIT_NS, 5);
    type Response = LevelInfo;
}
