use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, INVENTORY_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitItemInfo {
    pub index: i32,
    pub iid: String,
    pub name: String,
    pub kind: i32,
    pub endurance: i32,
    pub equipped: bool,
    // icon sprite name, served at /sprite/item/{icon}.png
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetUnitItemsRequest {
    pub force_id: i32,
    pub unit_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitItemsResponse {
    pub items: Vec<UnitItemInfo>,
}

impl Command for GetUnitItemsRequest {
    const ID: CommandId = CommandId::new(INVENTORY_NS, 0);
    type Response = UnitItemsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddItemRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub iid: String,
}

impl Command for AddItemRequest {
    const ID: CommandId = CommandId::new(INVENTORY_NS, 1);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveItemRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub item_index: i32,
}

impl Command for RemoveItemRequest {
    const ID: CommandId = CommandId::new(INVENTORY_NS, 2);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquipItemRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub item_index: i32,
}

impl Command for EquipItemRequest {
    const ID: CommandId = CommandId::new(INVENTORY_NS, 3);
    type Response = OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetEnduranceRequest {
    pub force_id: i32,
    pub unit_index: i32,
    pub item_index: i32,
    pub value: i32,
}

impl Command for SetEnduranceRequest {
    const ID: CommandId = CommandId::new(INVENTORY_NS, 4);
    type Response = OkResponse;
}
