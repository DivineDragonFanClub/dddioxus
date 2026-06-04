use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::SCRIPT_NS;
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptGlobal {
    pub key: String,
    pub kind: i32,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetScriptGlobalsRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetScriptGlobalsResponse {
    pub globals: Vec<ScriptGlobal>,
}

impl Command for GetScriptGlobalsRequest {
    const ID: CommandId = CommandId::new(SCRIPT_NS, 0);
    type Response = GetScriptGlobalsResponse;
}

pub fn data_type_label(kind: i32) -> &'static str {
    match kind {
        0 => "nil",
        1 => "void",
        2 => "bool",
        3 => "number",
        4 => "string",
        5 => "function",
        6 => "table",
        7 => "tuple",
        8 => "userdata",
        9 => "thread",
        _ => "?",
    }
}
