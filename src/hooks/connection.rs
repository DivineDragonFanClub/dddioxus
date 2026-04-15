use std::sync::Arc;

use dioxus::prelude::*;

use super::client::{Client, ServerInfo};

pub use super::client::{connect, discover_and_connect, ClientConfig};

#[derive(Clone)]
pub enum ConnectionState {
    Disconnected,
    Connected {
        client: Arc<Client>,
        info: ServerInfo,
    },
}

impl ConnectionState {
    pub fn is_open(&self) -> bool {
        matches!(self, ConnectionState::Connected { .. })
    }

    pub fn server_info(&self) -> Option<&ServerInfo> {
        match self {
            ConnectionState::Connected { info, .. } => Some(info),
            _ => None,
        }
    }

    pub fn client(&self) -> Option<&Arc<Client>> {
        match self {
            ConnectionState::Connected { client, .. } => Some(client),
            _ => None,
        }
    }
}

pub fn use_connection() -> Signal<ConnectionState> {
    let signal = use_signal(|| ConnectionState::Disconnected);
    use_hook(|| provide_context(signal))
}
