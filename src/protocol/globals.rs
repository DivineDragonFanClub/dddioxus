use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::VARIABLES_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalVariable {
    pub name: String,
    pub kind: String,
    pub value: String,
    pub temporary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGlobalVariablesRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGlobalVariablesResponse {
    pub variables: Vec<GlobalVariable>,
}

impl Command for GetGlobalVariablesRequest {
    const ID: CommandId = CommandId::new(VARIABLES_NS, 0);
    type Response = GetGlobalVariablesResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetGlobalVariableRequest {
    pub name: String,
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetGlobalVariableResponse {
    pub name: String,
    pub kind: String,
    pub value: String,
}

impl Command for SetGlobalVariableRequest {
    const ID: CommandId = CommandId::new(VARIABLES_NS, 1);
    type Response = SetGlobalVariableResponse;
}
