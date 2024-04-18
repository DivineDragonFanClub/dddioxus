use std::sync::{Arc, RwLock};
use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{Error, Message};
use crate::hooks::websocket::DebugConnection;
use crate::protocol::{RequestMessage, RmcRequestPacket, RmcResponsePacket};

#[derive(PartialEq, Clone, Props)]
pub struct MessageContextProviderProps {
    children: Element,
}

#[component]
pub fn MessageContextProvider(props: MessageContextProviderProps) -> Element {
    let mut ws = use_context::<Signal<DebugConnection>>();

    // Only provide the coroutine to child Elements if the WebSocket is open
    if ws().is_open() {
        println!("ws open");
        use_coroutine(|mut rx: UnboundedReceiver<RequestMessage>| async move {
            println!("use coroutine");
            while let Some(mut msg) = rx.next().await {
                let message = RmcRequestPacket {
                    call_id: 0,
                    method_id: msg.method_id,
                    params: msg.bytes,
                };

                    let message = Message::Binary(serde_json::to_vec(&message).unwrap());
                    ws.write().send(message).await;

                    if let Some(res) = ws().recv().await {
                        match res {
                            Ok(message) => {
                                match message {
                                    Message::Binary(resp) => {
                                        let response = serde_json::from_slice(&resp).unwrap();
                                        msg.sender.send(response).unwrap();
                                    }
                                    Message::Close(_) => {
                                        println!("close message");
                                    }
                                    _ => println!("other message")
                                }
                            }
                            Err(err) => {
                                match err {
                                    Error::ConnectionClosed | Error::Io(_) => {
                                        println!("ConnectionClosed or IO ");
                                        // TODO: Close the websocket somehow
                                        ws.write().close().await;
                                    }
                                    _ => println!("other error kind: {}", err)
                                }
                            }
                        }
                    }
                }

        });
    }

    rsx! {
        { props.children }
    }
}