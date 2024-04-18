use std::sync::{Arc, RwLock};
use dioxus::prelude::*;
use futures_util::stream::SplitSink;
use crate::hooks::DebugConnection;

#[derive(PartialEq, Clone, Props)]
pub struct WebsocketProviderProps {
    children: Element,
}



#[component]
pub fn WebSocketProvider(props: WebsocketProviderProps) -> Element {
    let mut url = use_signal(|| "ws://127.0.0.1:18050".to_string());

    // TODO: When this changes, we have to make it available to children props
    let mut ws = crate::hooks::use_ws_provider();

    let connect = move |_| {
        spawn(async move {
            // let mut ws = use_context::<DebugConnection>();
            println!("before connect");
            let socket = ws.write().connect(&url()).await;
            println!("after connect");

            match socket {
                Ok(connection) => {
                    println!("socket ok");
                },
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        });
    };

    rsx! {
        // Only display the children when the connection is established and open
        if ws().is_open() {
            { println!("socket open") }
            { props.children }
        } else {
            { println!("rendering provider") }
            input {
                value: "{url}",
                oninput: move |event| url.set(event.value()),
            }
            button { class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                onclick: connect,
                "Connect"
            }
        }
    }
}