use std::sync::Arc;

use dioxus::prelude::*;

use super::client::{Client, ServerInfo};

pub use super::client::{connect, query_server_port, watch_beacons, ClientConfig, DiscoveredServer};

#[derive(Clone)]
pub enum ConnectionState {
    Disconnected {
        reason: Option<String>,
    },
    Connected {
        client: Arc<Client>,
        info: ServerInfo,
    },
    /// The connection dropped but the workspace stays up; we keep watching
    /// beacons and reconnect when the server shows up again.
    Reconnecting {
        info: ServerInfo,
        reason: String,
    },
}

impl ConnectionState {
    pub fn client(&self) -> Option<&Arc<Client>> {
        match self {
            ConnectionState::Connected { client, .. } => Some(client),
            _ => None,
        }
    }

    pub fn disconnect_reason(&self) -> Option<&str> {
        match self {
            ConnectionState::Disconnected { reason } => reason.as_deref(),
            _ => None,
        }
    }
}

pub fn use_connection() -> Signal<ConnectionState> {
    let signal = use_signal(|| ConnectionState::Disconnected { reason: None });
    use_hook(|| provide_context(signal))
}
