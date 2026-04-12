use dioxus::prelude::*;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::{Error, Message};
use crate::hooks::websocket::{ConnectionState, DebugConnection, do_recv, do_send};
use crate::protocol::{RequestMessage, RmcRequestPacket};

#[derive(PartialEq, Clone, Props)]
pub struct MessageContextProviderProps {
    children: Element,
}

#[component]
pub fn MessageContextProvider(props: MessageContextProviderProps) -> Element {
    let mut ws = use_context::<Signal<DebugConnection>>();

    if ws().is_open() {
        use_coroutine(move |mut rx: UnboundedReceiver<RequestMessage>| async move {
            while let Some(msg) = rx.next().await {
                // Clone Arc handles out of the signal without holding a borrow across await
                let (sender, reader) = {
                    let conn = ws.read();
                    match (&conn.state, &conn.state) {
                        (ConnectionState::Connected { sender, .. }, ConnectionState::Connected { reader, .. }) => {
                            (sender.clone(), reader.clone())
                        }
                        _ => continue,
                    }
                };

                let packet = RmcRequestPacket {
                    call_id: 0,
                    method_id: msg.method_id,
                    params: msg.bytes,
                };

                let ws_msg = Message::Binary(serde_json::to_vec(&packet).unwrap());
                if !do_send(&sender, ws_msg).await {
                    ws.write().state = ConnectionState::Closed;
                    ws.write().is_open = false;
                    continue;
                }

                if let Some(res) = do_recv(&reader).await {
                    match res {
                        Ok(Message::Binary(resp)) => {
                            let response = serde_json::from_slice(&resp).unwrap();
                            let _ = msg.sender.send(response);
                        }
                        Ok(Message::Close(_)) => {
                            println!("Server sent close frame");
                            ws.write().state = ConnectionState::Closed;
                            ws.write().is_open = false;
                        }
                        Ok(_) => println!("Unexpected message type"),
                        Err(Error::ConnectionClosed | Error::Io(_)) => {
                            println!("Connection closed or IO error");
                            ws.write().state = ConnectionState::Closed;
                            ws.write().is_open = false;
                        }
                        Err(err) => println!("WebSocket error: {err}"),
                    }
                }
            }
        });
    }

    rsx! {
        { props.children }
    }
}