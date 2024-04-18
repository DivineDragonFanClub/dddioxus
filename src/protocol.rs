use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RmcRequestPacket {
    pub call_id: u32,
    pub method_id: u32,
    pub params: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RmcResponsePacket {
    pub call_id: u32,
    pub method_id: u32,
    pub params: Vec<u8>,
}

#[derive(Debug)]
pub struct RequestMessage {
    pub method_id: u32,
    pub bytes: Vec<u8>,
    pub sender: oneshot::Sender<RmcResponsePacket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSceneNameRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSceneNameResponse {
    pub scene_name: String,
}

pub trait RemoteMethodCall: Serialize + PartialEq + Clone + 'static {
    const METHOD_ID: u32;
    type Response: DeserializeOwned;
}

impl RemoteMethodCall for GetSceneNameRequest {
    const METHOD_ID: u32 = 0;
    type Response = GetSceneNameResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetProcTreeRequest;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetProcTreeResponse;

impl RemoteMethodCall for GetProcTreeRequest {
    const METHOD_ID: u32 = 1;
    type Response = GetProcTreeResponse;
}