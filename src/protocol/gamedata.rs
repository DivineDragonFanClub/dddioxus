use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::GAMEDATA_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassInfo {
    pub jid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetClassesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetClassesResponse {
    pub classes: Vec<ClassInfo>,
}

impl Command for GetClassesRequest {
    const ID: CommandId = CommandId::new(GAMEDATA_NS, 0);
    type Response = GetClassesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCatalogEntry {
    pub iid: String,
    pub name: String,
    pub kind: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetItemsRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetItemsResponse {
    pub items: Vec<ItemCatalogEntry>,
}

impl Command for GetItemsRequest {
    const ID: CommandId = CommandId::new(GAMEDATA_NS, 1);
    type Response = GetItemsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillCatalogEntry {
    pub sid: String,
    pub name: String,
    pub inheritable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSkillsRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSkillsResponse {
    pub skills: Vec<SkillCatalogEntry>,
}

impl Command for GetSkillsRequest {
    const ID: CommandId = CommandId::new(GAMEDATA_NS, 2);
    type Response = GetSkillsResponse;
}

pub const ITEM_KINDS: &[&str] = &[
    "None", "Sword", "Lance", "Axe", "Bow", "Dagger", "Magic", "Rod", "Fist", "Special", "Tool",
    "Shield", "Accessory", "Precious", "RefineIron", "RefineSteel", "RefineSilver", "PieceOfBond", "Gold",
];

pub fn item_kind_label(kind: i32) -> &'static str {
    ITEM_KINDS.get(kind as usize).copied().unwrap_or("Item")
}
