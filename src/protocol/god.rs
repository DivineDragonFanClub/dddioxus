use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::GOD_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BondInfo {
    pub gid: String,
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub max_level: i32,
    pub reliance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitBondsResponse {
    pub bonds: Vec<BondInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitBondsRequest {
    pub force_id: i32,
    pub unit_index: i32,
}

impl Command for GetUnitBondsRequest {
    const ID: CommandId = CommandId::new(GOD_NS, 0);
    type Response = GetUnitBondsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetBondLevelResponse {
    pub level: i32,
    pub exp: i32,
    pub reliance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetBondLevelRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub gid: String,
    pub level: i32,
}

impl Command for SetBondLevelRequest {
    const ID: CommandId = CommandId::new(GOD_NS, 1);
    type Response = SetBondLevelResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolderBond {
    pub pid: String,
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub current_level_exp: i32,
    pub next_level_exp: i32,
    pub max_level: i32,
    pub reliance: String,
    pub max_reliance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BondHolderInfo {
    pub gid: String,
    pub name: String,
    pub max_level: i32,
    pub a_rank_count: i32,
    pub bonds: Vec<HolderBond>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetBondHoldersResponse {
    pub holders: Vec<BondHolderInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetBondHoldersRequest;

impl Command for GetBondHoldersRequest {
    const ID: CommandId = CommandId::new(GOD_NS, 2);
    type Response = GetBondHoldersResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetHolderBondRequest {
    pub gid: String,
    pub pid: String,
    pub level: Option<i32>,
    pub exp: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetHolderBondResponse {
    pub level: i32,
    pub exp: i32,
    pub current_level_exp: i32,
    pub next_level_exp: i32,
    pub reliance: String,
}

impl Command for SetHolderBondRequest {
    const ID: CommandId = CommandId::new(GOD_NS, 3);
    type Response = SetHolderBondResponse;
}
