use std::sync::{Arc, RwLock};
use std::time::Duration;
use dioxus::dioxus_core::use_hook;
use dioxus::prelude::{provide_context, Signal, use_signal};

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{Error, Message};

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Closed,
    Connected {
        sender: Arc<RwLock<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        reader: Arc<RwLock<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    }
}

impl ConnectionState {
    fn is_open(&self) -> bool {
        matches!(self, ConnectionState::Connected { .. })
    }
}

/// Connect to the debug server. Retries with short timeouts because Ryujinx sucks ASS
/// and does not properly respect the opt arguments.
pub async fn do_connect(url: &str) -> std::io::Result<ConnectionState> {
    for attempt in 1..=10 {
        match tokio::time::timeout(Duration::from_secs(2), connect_async(url)).await {
            Ok(Ok((ws, _))) => {
                println!("Connected to {url} (attempt {attempt})");
                let (sender, reader) = ws.split();
                return Ok(ConnectionState::Connected {
                    sender: Arc::new(RwLock::new(sender)),
                    reader: Arc::new(RwLock::new(reader)),
                });
            }
            Ok(Err(e)) => {
                println!("Connection failed: {e}");
                return Err(std::io::Error::other(e));
            }
            Err(_) => {
                
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Could not reach the debug server. Is the game running?",
    ))
}

pub async fn do_send(
    sender: &Arc<RwLock<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    msg: Message,
) -> bool {
    sender.write().unwrap().send(msg).await.is_ok()
}

pub async fn do_recv(
    reader: &Arc<RwLock<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
) -> Option<Result<Message, Error>> {
    reader.write().unwrap().next().await
}

#[derive(Clone)]
pub struct DebugConnection {
    pub state: ConnectionState,
    pub is_open: bool,
}

impl DebugConnection {
    pub fn is_open(&self) -> bool {
        self.state.is_open()
    }

    pub fn get_sender(&self) -> Option<Arc<RwLock<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>> {
        match &self.state {
            ConnectionState::Connected { sender, .. } => Some(sender.clone()),
            _ => None,
        }
    }

    pub fn get_reader(&self) -> Option<Arc<RwLock<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>> {
        match &self.state {
            ConnectionState::Connected { reader, .. } => Some(reader.clone()),
            _ => None,
        }
    }
}

pub fn use_ws_provider() -> Signal<DebugConnection> {
    let signal = use_signal(|| {
        DebugConnection {
            state: ConnectionState::Closed,
            is_open: false,
        }
    });

    use_hook(|| {
        provide_context(signal)
    })
}
