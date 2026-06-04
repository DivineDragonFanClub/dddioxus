use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::CUTSCENE_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutsceneStep {
    pub index: i32,
    pub label: String,
    pub dialogue: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCutsceneResponse {
    pub active: bool,
    pub demo_name: String,
    pub mess_file: String,
    pub current_label: String,
    pub current_index: i32,
    pub steps: Vec<CutsceneStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCutsceneRequest;

impl Command for GetCutsceneRequest {
    const ID: CommandId = CommandId::new(CUTSCENE_NS, 0);
    type Response = GetCutsceneResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JumpCutsceneResponse {
    pub current_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JumpCutsceneRequest {
    pub index: i32,
}

impl Command for JumpCutsceneRequest {
    const ID: CommandId = CommandId::new(CUTSCENE_NS, 1);
    type Response = JumpCutsceneResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditCommandRequest {
    pub label: String,
    pub side: String,
    pub index: i32,
    pub text: String,
}

impl Command for EditCommandRequest {
    const ID: CommandId = CommandId::new(CUTSCENE_NS, 2);
    type Response = crate::protocol::OkResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveCommandRequest {
    pub label: String,
    pub side: String,
    pub index: i32,
}

impl Command for RemoveCommandRequest {
    const ID: CommandId = CommandId::new(CUTSCENE_NS, 3);
    type Response = crate::protocol::OkResponse;
}
