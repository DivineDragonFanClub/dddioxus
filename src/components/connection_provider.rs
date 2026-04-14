use dioxus::prelude::*;

use crate::hooks::connection::{
    discover_and_connect, ClientConfig, ConnectionState,
};

#[derive(PartialEq, Clone, Props)]
pub struct ConnectionProviderProps {
    #[props(default)]
    config: ClientConfig,
    children: Element,
}

#[component]
pub fn ConnectionProvider(props: ConnectionProviderProps) -> Element {
    let mut connecting = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut conn = crate::hooks::connection::use_connection();
    let config = props.config.clone();

    let connect = move |_| {
        if connecting() {
            return;
        }
        connecting.set(true);
        error_msg.set(None);

        let cfg = config.clone();
        spawn(async move {
            match discover_and_connect(&cfg).await {
                Ok(client) => {
                    let info = client.info().clone();
                    conn.set(ConnectionState::Connected { client, info });
                }
                Err(err) => error_msg.set(Some(err)),
            }
            connecting.set(false);
        });
    };

    let disconnect = move |_| {
        conn.set(ConnectionState::Disconnected);
    };

    let is_connected = conn.read().is_open();
    let server_info = conn.read().server_info().cloned();

    if let (true, Some(info)) = (is_connected, server_info) {
        let conn_key = format!("{}:{}", info.host, info.port);
        rsx! {
            div { class: "flex flex-col h-full",
                div { class: "flex items-center justify-between px-4 py-2 bg-gray-800 text-sm text-gray-300",
                    span { "Connected to {info.host}:{info.port} (v{info.api_version})" }
                    button {
                        class: "text-red-400 hover:text-red-300 text-xs",
                        onclick: disconnect,
                        "Disconnect"
                    }
                }
                div { key: "{conn_key}", class: "flex flex-1 overflow-hidden", { props.children } }
            }
        }
    } else {
        rsx! {
            div { class: "flex flex-col items-center gap-4 p-8",
                p { class: "text-gray-400 text-sm",
                    "Searching for server on the local network..."
                }
                button {
                    class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                    disabled: connecting(),
                    onclick: connect,
                    if connecting() {
                        "Discovering..."
                    } else {
                        "Connect"
                    }
                }
                if let Some(err) = error_msg() {
                    p { class: "text-red-500 text-sm", "{err}" }
                }
            }
        }
    }
}
