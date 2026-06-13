use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, GMAP_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterInfo {
    pub rank: i32,
    // 0 none, 1 exp (gold mob), 2 gold (silver mob)
    pub rare: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmapNode {
    pub cid: String,
    pub name: String,
    pub chapter: String,
    pub state: i32,
    pub has_encounter: bool,
    pub encounter: Option<EncounterInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGmapNodesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGmapNodesResponse {
    pub available: bool,
    pub nodes: Vec<GmapNode>,
}

impl Command for GetGmapNodesRequest {
    const ID: CommandId = CommandId::new(GMAP_NS, 0);
    type Response = GetGmapNodesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetNodeStateRequest {
    pub cid: String,
    pub state: i32,
}

impl Command for SetNodeStateRequest {
    const ID: CommandId = CommandId::new(GMAP_NS, 1);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnEncounterRequest {
    pub cid: String,
}

impl Command for SpawnEncounterRequest {
    const ID: CommandId = CommandId::new(GMAP_NS, 2);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DespawnEncounterRequest {
    pub cid: String,
}

impl Command for DespawnEncounterRequest {
    const ID: CommandId = CommandId::new(GMAP_NS, 3);
    type Response = OkResponse;
}
