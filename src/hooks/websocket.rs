use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use dioxus::dioxus_core::use_hook;
use dioxus::hooks::use_context_provider;
use dioxus::prelude::{provide_context, schedule_update, Signal, use_signal};

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
    async fn connect(&mut self, url: impl AsRef<str>) -> std::io::Result<()> {
        match self {
            ConnectionState::Closed => {
                match connect_async(url.as_ref()).await {
                    Ok((ws, _)) => {
                        let (sender, reader) = ws.split();

                        *self = ConnectionState::Connected {
                            sender: Arc::new(RwLock::new(sender)),
                            reader: Arc::new(RwLock::new(reader)),
                        };

                        println!("Connected");

                        Ok(())
                    }
                    Err(_) => Err(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable)),
                }
            }
            ConnectionState::Connected { sender, .. } => {
                self.close().await;
                Ok(())
            }
        }
    }

    async fn send(&mut self, msg: Message) -> Option<()> {
        match self {
            ConnectionState::Closed => {
                // Probably return a error here to signal the websocket isn't open
                None
            }
            ConnectionState::Connected { sender, .. } => {
                // TODO: Change the state to Closed on error
                let result = sender.write().unwrap().send(msg).await;

                if let Err(err) = result {
                    *self = ConnectionState::Closed;
                    None
                } else {
                    Some(())
                }
            }
        }
    }

    async fn recv(&self) -> Option<Result<Message, Error>> {
        match self {
            ConnectionState::Closed => {
                // Probably return a error here to signal the websocket isn't open
                None
            }
            ConnectionState::Connected { reader, .. } => {
                reader.write().unwrap().next().await
            }
        }
    }

    fn is_open(&self) -> bool {
        match self {
            ConnectionState::Closed => false,
            ConnectionState::Connected { .. } => true
        }
    }

    async fn close(&mut self) {
        match self {
            ConnectionState::Closed => {}
            ConnectionState::Connected { sender, .. } => {
                sender.write().unwrap().send(Message::Close(None)).await;
                *self = ConnectionState::Closed;
            }
        }
    }
}

#[derive(Clone)]
pub struct DebugConnection {
    state: ConnectionState,
    update: Arc<dyn Fn()>,
    is_open: bool,
}

// impl DebugConnection {
//     pub fn new() -> Self {
//         Self {
//             state: ConnectionState::Closed,
//         }
//     }
// }

impl DebugConnection {
    pub async fn connect(&mut self, url: &str) -> std::io::Result<()> {
        let result = self.state.connect(url).await;

        self.is_open = self.state.is_open();

        result
    }

    pub fn is_open(&self) -> bool {
        self.state.is_open()
    }

    pub async fn recv(&self) -> Option<Result<Message, Error>> {
        self.state.recv().await
    }

    pub async fn send(&mut self, msg: Message) -> Option<()> {
        self.state.send(msg).await
    }

    pub async fn close(&mut self) {
        self.state.close().await
    }
}

pub fn use_ws_provider() -> Signal<DebugConnection> {
    let update = schedule_update();


    let signal = use_signal(|| {
        println!("Creating signal");

        DebugConnection {
        state: ConnectionState::Closed,
        update,
            is_open: false
    }});

    use_hook(|| {
        println!("provide_context");

        provide_context(signal)
    })
}