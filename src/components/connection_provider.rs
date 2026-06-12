use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;

use crate::hooks::connection::{connect, watch_beacons, ClientConfig, ConnectionState, DiscoveredServer};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(PartialEq, Clone, Props)]
pub struct ConnectionProviderProps {
    #[props(default)]
    config: ClientConfig,
    children: Element,
}

#[component]
pub fn ConnectionProvider(props: ConnectionProviderProps) -> Element {
    let mut servers: Signal<Vec<DiscoveredServer>> = use_signal(Vec::new);
    let mut connecting_to: Signal<Option<DiscoveredServer>> = use_signal(|| None);
    let mut connect_error: Signal<Option<String>> = use_signal(|| None);
    let mut listen_error: Signal<Option<String>> = use_signal(|| None);
    let mut conn = use_context::<Signal<ConnectionState>>();
    let config = props.config.clone();

    // Listen for server beacons for the whole lifetime of the app.
    let watch_config = config.clone();
    use_hook(move || {
        spawn(async move {
            let mut rx = watch_beacons(&watch_config);
            while let Some(update) = rx.recv().await {
                match update {
                    Ok(list) => {
                        servers.set(list);
                        listen_error.set(None);
                    }
                    Err(err) => listen_error.set(Some(err)),
                }
            }
        });
    });

    let mut retrying = use_signal(|| false);

    use_effect(move || {
        if let Some(client) = conn.read().client().cloned() {
            spawn(async move {
                let reason = client.wait_disconnect().await;
                // Only react if this client is still the active one — a
                // session we already left shouldn't disturb a newer
                // connection.
                let is_current = conn.peek().client().is_some_and(|c| Arc::ptr_eq(c, &client));
                if is_current {
                    // Keep the workspace up; the user reconnects from the
                    // banner once the server is back.
                    let info = client.info().clone();
                    conn.set(ConnectionState::Reconnecting { info, reason });
                }
            });
        }
    });

    let retry_config = props.config.clone();
    let retry = use_callback(move |_: ()| {
        if retrying() {
            return;
        }
        let target_info = match &*conn.peek() {
            ConnectionState::Reconnecting { info, .. } => info.clone(),
            _ => return,
        };
        retrying.set(true);
        let cfg = retry_config.clone();
        spawn(async move {
            // A beacon tells us where the server is now (the port changes
            // when the game relaunches); without one, try the last known
            // address directly.
            let target = servers
                .peek()
                .iter()
                .filter(|s| s.host == target_info.host)
                .max_by_key(|s| s.port == target_info.port)
                .cloned()
                .unwrap_or(DiscoveredServer {
                    host: target_info.host.clone(),
                    port: target_info.port,
                });
            if let Ok(Ok(client)) =
                tokio::time::timeout(CONNECT_TIMEOUT, connect(&target.host, target.port, &cfg)).await
            {
                // The user may have gone back to the server list while the
                // attempt was in flight.
                if matches!(&*conn.peek(), ConnectionState::Reconnecting { .. }) {
                    let info = client.info().clone();
                    conn.set(ConnectionState::Connected { client, info });
                } else {
                    client.close().await;
                }
            }
            retrying.set(false);
        });
    });

    let connect_to = use_callback(move |server: DiscoveredServer| {
        if connecting_to().is_some() {
            return;
        }
        connecting_to.set(Some(server.clone()));
        connect_error.set(None);
        let cfg = config.clone();
        spawn(async move {
            match tokio::time::timeout(CONNECT_TIMEOUT, connect(&server.host, server.port, &cfg)).await {
                Ok(Ok(client)) => {
                    let info = client.info().clone();
                    conn.set(ConnectionState::Connected { client, info });
                }
                Ok(Err(err)) => connect_error.set(Some(err)),
                Err(_) => connect_error.set(Some(format!(
                    "Timed out connecting to {}:{}",
                    server.host, server.port
                ))),
            }
            connecting_to.set(None);
        });
    });

    // Connected and Reconnecting both keep the workspace mounted so panel
    // state survives a transient drop; only the header bar changes.
    let workspace = match &*conn.read() {
        ConnectionState::Connected { info, .. } => Some((info.clone(), None)),
        ConnectionState::Reconnecting { info, reason } => Some((info.clone(), Some(reason.clone()))),
        ConnectionState::Disconnected { .. } => None,
    };
    let disconnect_reason = conn.read().disconnect_reason().map(|s| s.to_string());

    if let Some((info, reconnecting)) = workspace {
        let conn_key = format!("{}:{}", info.host, info.port);
        // Dioxus ignores keys on nested elements — for a key change to
        // remount the workspace (fresh panels after reconnecting to a
        // relaunched game on a new port) the keyed div must be the root of
        // its own rsx block, embedded below as a dynamic node.
        let workspace_body = rsx! {
            div {
                key: "{conn_key}",
                class: "flex flex-1 overflow-hidden min-h-0",
                {props.children}
            }
        };
        rsx! {
            div {
                "data-component": "ConnectionProvider",
                class: "flex flex-col h-full",
                if let Some(reason) = reconnecting {
                    div { class: "flex items-center justify-between px-4 py-2 bg-yellow-900 text-sm text-yellow-200 flex-shrink-0",
                        div { class: "flex items-center gap-2.5 min-w-0",
                            span { class: "relative flex h-2.5 w-2.5 flex-shrink-0",
                                span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-yellow-400 opacity-75" }
                                span { class: "relative inline-flex rounded-full h-2.5 w-2.5 bg-yellow-500" }
                            }
                            span { class: "truncate", "Connection lost ({reason})" }
                        }
                        div { class: "flex items-center gap-3 flex-shrink-0",
                            button {
                                class: "text-yellow-100 bg-yellow-800 hover:bg-yellow-700 text-xs px-2 py-0.5 rounded",
                                disabled: retrying(),
                                onclick: move |_| retry.call(()),
                                if retrying() { "Reconnecting..." } else { "⟳ Retry" }
                            }
                            button {
                                class: "text-yellow-300 hover:text-yellow-100 text-xs underline",
                                onclick: move |_| conn.set(ConnectionState::Disconnected { reason: None }),
                                "Back to servers"
                            }
                        }
                    }
                } else {
                    div { class: "flex items-center justify-between px-4 py-2 bg-gray-800 text-sm text-gray-300 flex-shrink-0",
                        span { "Connected to {info.host}:{info.port} (v{info.api_version})" }
                        button {
                            class: "text-red-400 hover:text-red-300 text-xs",
                            onclick: move |_| {
                                let old = conn.peek().client().cloned();
                                conn.set(ConnectionState::Disconnected {
                                    reason: None,
                                });
                                if let Some(client) = old {
                                    spawn(async move { client.close().await });
                                }
                            },
                            "Disconnect"
                        }
                    }
                }
                {workspace_body}
            }
        }
    } else {
        let is_connecting = connecting_to().is_some();
        // The static Tailwind v2 stylesheet has no `disabled:` variants, so
        // branch the row styling on state instead.
        let row_class = if is_connecting {
            "flex items-center justify-between w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded text-left opacity-50"
        } else {
            "flex items-center justify-between w-full px-4 py-3 bg-gray-800 border border-gray-700 hover:border-indigo-500 rounded text-left"
        };
        rsx! {
            div { class: "flex flex-col items-center gap-6 p-8",
                if let Some(reason) = disconnect_reason {
                    div { class: "w-full max-w-md px-4 py-3 bg-red-900/40 border border-red-700 rounded text-center",
                        p { class: "text-red-300 text-sm font-semibold", "Connection lost" }
                        p { class: "text-red-200 text-xs mt-1", "{reason}" }
                    }
                }
                if listen_error().is_none() {
                    div { class: "flex items-center gap-2.5",
                        span { class: "relative flex h-2.5 w-2.5",
                            span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75" }
                            span { class: "relative inline-flex rounded-full h-2.5 w-2.5 bg-indigo-500" }
                        }
                        p { class: "text-gray-400 text-sm",
                            if servers().is_empty() {
                                "Searching for debug servers on your local network..."
                            } else {
                                "Click on your server to connect"
                            }
                        }
                    }
                }
                div { class: "flex flex-col gap-2 w-full max-w-md p-4 border border-gray-700 rounded-lg",
                    if servers().is_empty() {
                        p { class: "text-gray-600 text-sm text-center py-3",
                            "Launch the game and the server will show up here."
                        }
                    }
                    for server in servers() {
                        button {
                            key: "{server.host}:{server.port}",
                            class: row_class,
                            disabled: is_connecting,
                            onclick: {
                                let server = server.clone();
                                move |_| connect_to.call(server.clone())
                            },
                            span { class: "font-mono text-sm text-white", "{server.host}:{server.port}" }
                            span { class: "text-xs text-indigo-400", "Connect" }
                        }
                    }
                }
                // Rendered outside the list so it survives the row being
                // pruned mid-attempt.
                if let Some(target) = connecting_to() {
                    p { class: "text-indigo-400 text-sm",
                        "Connecting to {target.host}:{target.port}..."
                    }
                }
                if let Some(err) = listen_error() {
                    p { class: "text-yellow-500 text-sm max-w-md text-center", "{err}" }
                }
                if let Some(err) = connect_error() {
                    p { class: "text-red-500 text-sm max-w-md text-center", "{err}" }
                }
            }
        }
    }
}
