use dioxus::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sora_protocol::command::CommandId;

use crate::hooks::connection::ConnectionState;

pub trait Command: Serialize + PartialEq + Clone + 'static {
    const ID: CommandId;
    type Response: DeserializeOwned;
}

pub async fn call<C: Command>(
    conn: &Signal<ConnectionState>,
    request: C,
) -> Result<C::Response, String> {
    let client = match conn.read().client() {
        Some(c) => c.clone(),
        None => return Err("Not connected".into()),
    };
    client.call::<C, C::Response>(request, C::ID).await
}
