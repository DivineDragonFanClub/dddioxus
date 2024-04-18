use dioxus::hooks::{Resource, use_coroutine_handle, use_reactive, use_resource};
use tokio::sync::oneshot;
use crate::protocol::{RemoteMethodCall, RequestMessage};
use futures_util::{SinkExt, StreamExt};


pub fn use_command<RMC: RemoteMethodCall>(msg: RMC) -> Resource<Option<RMC::Response>> {
    println!("command handle before");
    let handle = use_coroutine_handle::<RequestMessage>();
    println!("command handle after");


    use_resource(use_reactive!(|(msg)| async move {
        let (tx, mut rx) = oneshot::channel();

        handle.send(RequestMessage {
            method_id: RMC::METHOD_ID,
            bytes: serde_json::ser::to_vec(&msg).unwrap(),
            sender: tx,
        });

        rx.await.ok().map(|response| serde_json::from_slice(&response.params).unwrap())
    }))
}