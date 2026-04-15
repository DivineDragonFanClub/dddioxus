use std::fs;
use std::path::PathBuf;

use dioxus::prelude::*;

use crate::hooks::connection::{connect, discover_and_connect, ClientConfig, ConnectionState};

/// Path to the dotfile that stores the last successful manual host:port.
/// Lives in $HOME so it survives `dx build`/`cargo clean`/app reinstall.
fn last_host_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".dddioxus_last_host"))
}

fn load_last_host() -> String {
    last_host_path()
        .and_then(|p| fs::read_to_string(&p).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn save_last_host(value: &str) {
    if let Some(path) = last_host_path() {
        let _ = fs::write(&path, value.trim());
    }
}

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
    let mut manual_input = use_signal(load_last_host);
    let mut conn = use_context::<Signal<ConnectionState>>();
    let config = props.config.clone();
    let manual_config = config.clone();

    let auto_connect = move |_| {
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

    let manual_connect = use_callback(move |_: ()| {
        if connecting() {
            return;
        }
        let raw = manual_input().trim().to_string();
        if raw.is_empty() {
            error_msg.set(Some("Enter host:port (e.g. 10.0.0.120:58391)".into()));
            return;
        }
        let (host, port) = match raw.rsplit_once(':') {
            Some((h, p)) => match p.parse::<u16>() {
                Ok(port) => (h.to_string(), port),
                Err(_) => {
                    error_msg.set(Some(format!("Invalid port: {p}")));
                    return;
                }
            },
            None => {
                error_msg.set(Some("Expected host:port".into()));
                return;
            }
        };
        connecting.set(true);
        error_msg.set(None);
        let cfg = manual_config.clone();
        let saved = raw;
        spawn(async move {
            match connect(&host, port, &cfg).await {
                Ok(client) => {
                    let info = client.info().clone();
                    save_last_host(&saved);
                    conn.set(ConnectionState::Connected { client, info });
                }
                Err(err) => error_msg.set(Some(err)),
            }
            connecting.set(false);
        });
    });

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
            div { class: "flex flex-col items-center gap-6 p-8",
                div { class: "flex flex-col items-center gap-3",
                    p { class: "text-gray-400 text-sm",
                        "Searching for server on the local network..."
                    }
                    button {
                        class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                        disabled: connecting(),
                        onclick: auto_connect,
                        if connecting() {
                            "Discovering..."
                        } else {
                            "Auto-discover"
                        }
                    }
                }
                div { class: "flex flex-col items-center gap-2 w-80",
                    p { class: "text-gray-500 text-xs uppercase tracking-wide", "or connect manually" }
                    input {
                        class: "w-full px-3 py-2 bg-gray-800 text-white rounded border border-gray-700 focus:border-indigo-500 focus:outline-none text-sm font-mono",
                        placeholder: "host:port (e.g. 10.0.0.120:58391)",
                        value: "{manual_input}",
                        oninput: move |e| manual_input.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                manual_connect.call(());
                            }
                        },
                    }
                    button {
                        class: "text-white bg-gray-700 border-0 py-1.5 px-4 focus:outline-none hover:bg-gray-600 rounded text-sm w-full",
                        disabled: connecting(),
                        onclick: move |_| manual_connect.call(()),
                        if connecting() { "Connecting..." } else { "Manual connect" }
                    }
                }
                if let Some(err) = error_msg() {
                    p { class: "text-red-500 text-sm max-w-md text-center", "{err}" }
                }
            }
        }
    }
}
