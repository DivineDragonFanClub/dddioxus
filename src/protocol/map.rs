use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, MAP_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapStatusRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapStatusResponse {
    pub in_map: bool,
}

impl Command for MapStatusRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 0);
    type Response = MapStatusResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapGridRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapGridResponse {
    pub width: i32,
    pub height: i32,
}

impl Command for MapGridRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 1);
    type Response = MapGridResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapUnit {
    pub x: i32,
    pub z: i32,
    pub force_id: i32,
    pub unit_index: i32,
    pub name: String,
    pub level: i32,
    pub class_jid: String,
    pub acted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapPlacementsRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapPlacementsResponse {
    pub units: Vec<MapUnit>,
}

impl Command for MapPlacementsRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 2);
    type Response = MapPlacementsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapTurnRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapTurnResponse {
    pub turn: i32,
}

impl Command for MapTurnRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 3);
    type Response = MapTurnResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetMapTurnRequest {
    pub turn: i32,
}

impl Command for SetMapTurnRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 4);
    type Response = MapTurnResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUnitPosRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub x: i32,
    pub z: i32,
}

impl Command for SetUnitPosRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 5);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompleteMapRequest;

impl Command for CompleteMapRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 6);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindEntry {
    pub index: i32,
    pub actor: String,
    pub action: String,
    pub is_phase_begin: bool,
    pub force: i32,
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindEntriesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindEntriesResponse {
    pub entries: Vec<RewindEntry>,
}

impl Command for RewindEntriesRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 7);
    type Response = RewindEntriesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindPreviewRequest {
    pub index: i32,
}

impl Command for RewindPreviewRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 8);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindCommitRequest;

impl Command for RewindCommitRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 9);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindCancelRequest;

impl Command for RewindCancelRequest {
    const ID: CommandId = CommandId::new(MAP_NS, 10);
    type Response = OkResponse;
}
