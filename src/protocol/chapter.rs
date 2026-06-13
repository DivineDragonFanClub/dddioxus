use serde::{Deserialize, Serialize};
use sora_protocol::command::CommandId;

use super::{OkResponse, CHAPTER_NS};
use crate::rpc::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterInfo {
    pub cid: String,
    pub title: String,
    pub kind: String,
    pub story: bool,
    pub cleared: bool,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetChaptersRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetChaptersResponse {
    pub available: bool,
    pub current_cid: String,
    pub progress: i32,
    pub chapters: Vec<ChapterInfo>,
}

impl Command for GetChaptersRequest {
    const ID: CommandId = CommandId::new(CHAPTER_NS, 0);
    type Response = GetChaptersResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCurrentChapterRequest {
    pub cid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCurrentChapterResponse {
    pub reset: i32,
}

impl Command for SetCurrentChapterRequest {
    const ID: CommandId = CommandId::new(CHAPTER_NS, 1);
    type Response = SetCurrentChapterResponse;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetChapterClearedRequest {
    pub cid: String,
    pub cleared: bool,
}

impl Command for SetChapterClearedRequest {
    const ID: CommandId = CommandId::new(CHAPTER_NS, 2);
    type Response = OkResponse;
}
