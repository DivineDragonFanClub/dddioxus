use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::MESS_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessFileInfo {
    pub name: String,
    pub reference_count: i32,
    pub label_count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMessFilesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMessFilesResponse {
    pub files: Vec<MessFileInfo>,
}

impl Command for ListMessFilesRequest {
    const ID: CommandId = CommandId::new(MESS_NS, 0);
    type Response = ListMessFilesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessEntry {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMessLabelsRequest {
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMessLabelsResponse {
    pub file: String,
    pub entries: Vec<MessEntry>,
}

impl Command for GetMessLabelsRequest {
    const ID: CommandId = CommandId::new(MESS_NS, 1);
    type Response = GetMessLabelsResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetMessTextRequest {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetMessTextResponse {
    pub text: String,
}

impl Command for SetMessTextRequest {
    const ID: CommandId = CommandId::new(MESS_NS, 2);
    type Response = SetMessTextResponse;
}
