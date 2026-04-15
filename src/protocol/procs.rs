use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::PROCS_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcNode {
    pub name: String,
    pub hashcode: i32,
    pub desc_index: i32,
    pub children: Vec<ProcNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcRoot {
    pub label: String,
    pub children: Vec<ProcNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetProcTreeRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetProcTreeResponse {
    pub roots: Vec<ProcRoot>,
}

impl Command for GetProcTreeRequest {
    const ID: CommandId = CommandId::new(PROCS_NS, 0);
    type Response = GetProcTreeResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetProcDescsRequest {
    pub root: String,
    pub path: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcDescInfo {
    pub kind: String,
    pub method: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetProcDescsResponse {
    pub descs: Vec<ProcDescInfo>,
    pub desc_index: i32,
}

impl Command for GetProcDescsRequest {
    const ID: CommandId = CommandId::new(PROCS_NS, 1);
    type Response = GetProcDescsResponse;
}
