//! Transport + handshake + call multiplexing. No Dioxus types live here so this
//! module can be lifted into its own `sora-client` crate later without changes.
//!
//! TODO once a second codec or compression/encryption impl exists:
//! - Swap `Arc<JsonCodec>` for `Arc<dyn FrameCodec>` (needs a dyn-safe wrapper
//!   around `sora_protocol::codec::Codec`, likely via `erased_serde`).
//! - Let `ClientBuilder` register multiple codecs + compression + encryption
//!   implementations; the builder then picks what to advertise in the handshake.

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sora_codecs::JsonCodec;
use sora_protocol::codec::Codec;
use sora_protocol::command::CommandId;
use sora_protocol::frame::Frame;
use sora_protocol::handshake::{
    ApiVersion, CodecId, CompressionId, EncryptionId, Handshake, HandshakeAck, HandshakeStatus,
};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex as AsyncMutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const BEACON_MAGIC: &[u8; 4] = b"OZN\x01";

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Clone, PartialEq)]
pub struct ClientConfig {
    pub api_version: ApiVersion,
    pub codec: CodecId,
    pub compression: CompressionId,
    pub encryption: EncryptionId,
    pub beacon_port: u16,
    pub beacon_timeout: Duration,
    pub beacon_attempts: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_version: ApiVersion::new(0, 1, 0),
            codec: CodecId::Json,
            compression: CompressionId::None,
            encryption: EncryptionId::None,
            beacon_port: 18051,
            beacon_timeout: Duration::from_secs(3),
            beacon_attempts: 5,
        }
    }
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder(Self::default())
    }
}

pub struct ClientConfigBuilder(ClientConfig);

impl ClientConfigBuilder {
    pub fn api_version(mut self, v: ApiVersion) -> Self {
        self.0.api_version = v;
        self
    }
    pub fn codec(mut self, c: CodecId) -> Self {
        self.0.codec = c;
        self
    }
    pub fn compression(mut self, c: CompressionId) -> Self {
        self.0.compression = c;
        self
    }
    pub fn encryption(mut self, e: EncryptionId) -> Self {
        self.0.encryption = e;
        self
    }
    pub fn beacon_port(mut self, port: u16) -> Self {
        self.0.beacon_port = port;
        self
    }
    pub fn beacon_timeout(mut self, timeout: Duration) -> Self {
        self.0.beacon_timeout = timeout;
        self
    }
    pub fn beacon_attempts(mut self, attempts: u32) -> Self {
        self.0.beacon_attempts = attempts;
        self
    }
    pub fn build(self) -> ClientConfig {
        self.0
    }
}

#[derive(Clone)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    pub api_version: ApiVersion,
}

pub struct Client {
    codec: Arc<JsonCodec>,
    sink: AsyncMutex<WsSink>,
    pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Frame>>>>,
    next_call_id: AtomicU32,
    info: ServerInfo,
}

impl Client {
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    pub async fn call<Req, Resp>(&self, request: Req, cmd_id: CommandId) -> Result<Resp, String>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let payload = self.codec.encode(&request).map_err(|e| format!("Serialize: {e}"))?;
        let response = self.call_raw(cmd_id, payload).await?;
        match response {
            Frame::Response { payload, .. } => {
                self.codec.decode(&payload).map_err(|e| format!("Deserialize: {e}"))
            }
            Frame::Error { detail, module, code, .. } => {
                Err(format!("[{module}-{code:04}] {detail}"))
            }
            _ => Err("Unexpected frame".into()),
        }
    }

    async fn call_raw(&self, cmd: CommandId, payload: Vec<u8>) -> Result<Frame, String> {
        let call_id = self.next_call_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(call_id, tx);

        let frame = Frame::request(call_id, cmd.namespace, cmd.command, payload);
        let bytes = self.codec.encode(&frame).map_err(|e| format!("Encode: {e}"))?;

        {
            let mut sink = self.sink.lock().await;
            if let Err(e) = sink.send(Message::Binary(bytes)).await {
                self.pending.lock().unwrap().remove(&call_id);
                return Err(format!("Send: {e}"));
            }
        }

        rx.await.map_err(|_| "Connection closed".to_string())
    }
}

pub async fn discover(config: &ClientConfig) -> Result<(String, u16), String> {
    let beacon_port = config.beacon_port;
    let timeout = config.beacon_timeout;
    let attempts = config.beacon_attempts;

    tokio::task::spawn_blocking(move || {
        let socket = UdpSocket::bind(format!("0.0.0.0:{beacon_port}"))
            .map_err(|e| format!("Failed to bind beacon port {beacon_port}: {e}"))?;
        socket
            .set_read_timeout(Some(timeout))
            .map_err(|e| format!("Failed to set beacon timeout: {e}"))?;

        let mut buf = [0u8; 64];
        for _ in 0..attempts {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    if len >= 6 && &buf[..4] == BEACON_MAGIC {
                        let port = u16::from_le_bytes([buf[4], buf[5]]);
                        log::info!("Beacon from {src}: server port = {port}");
                        return Ok((src.ip().to_string(), port));
                    }
                }
                Err(_) => continue,
            }
        }
        Err(format!("No server found after {attempts} discovery attempts"))
    })
    .await
    .unwrap_or_else(|e| Err(format!("Beacon task failed: {e}")))
}

pub async fn connect(host: &str, port: u16, config: &ClientConfig) -> Result<Arc<Client>, String> {
    let url = format!("ws://{host}:{port}");
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {e}"))?;

    let (sink, mut stream) = ws.split();
    let sink = AsyncMutex::new(sink);

    let hs = Handshake {
        client_api_version: config.api_version,
        codec_id: config.codec as u8,
        compression_id: config.compression as u8,
        encryption_id: config.encryption as u8,
        encryption_params: vec![],
    };

    {
        let mut s = sink.lock().await;
        s.send(Message::Binary(hs.to_bytes()))
            .await
            .map_err(|e| format!("Failed to send handshake: {e}"))?;
    }

    let ack_data = loop {
        match stream.next().await {
            Some(Ok(Message::Binary(data))) => break data,
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(format!("Failed to receive handshake ack: {e}")),
            None => return Err("Connection closed during handshake".into()),
        }
    };

    let ack = HandshakeAck::from_bytes(&ack_data).map_err(|e| format!("Invalid handshake ack: {e}"))?;
    match ack.status {
        HandshakeStatus::Accepted => {
            log::info!("Connected to server v{}", ack.server_api_version);
        }
        HandshakeStatus::Rejected => {
            let reason = ack.rejection_reason().unwrap_or("unknown");
            return Err(format!("Server rejected connection: {reason}"));
        }
    }

    let codec = Arc::new(JsonCodec);
    let pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Frame>>>> = Arc::new(Mutex::new(HashMap::new()));

    spawn_reader(stream, codec.clone(), pending.clone());

    Ok(Arc::new(Client {
        codec,
        sink,
        pending,
        next_call_id: AtomicU32::new(1),
        info: ServerInfo {
            host: host.to_string(),
            port,
            api_version: ack.server_api_version,
        },
    }))
}

pub async fn discover_and_connect(config: &ClientConfig) -> Result<Arc<Client>, String> {
    let (host, port) = discover(config).await?;
    connect(&host, port, config).await
}

fn spawn_reader(
    mut stream: WsStream,
    codec: Arc<JsonCodec>,
    pending: Arc<Mutex<HashMap<u32, oneshot::Sender<Frame>>>>,
) {
    tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            let bytes = match msg {
                Ok(Message::Binary(b)) => b,
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(_) => continue,
            };

            let frame: Frame = match codec.decode(&bytes) {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("Failed to decode frame: {e}");
                    continue;
                }
            };

            let call_id = match &frame {
                Frame::Response { call_id, .. } => *call_id,
                Frame::Error { call_id, .. } => *call_id,
                _ => continue,
            };

            if let Some(tx) = pending.lock().unwrap().remove(&call_id) {
                let _ = tx.send(frame);
            }
        }

        // Connection closed — fail every outstanding call so callers can surface the error.
        let mut map = pending.lock().unwrap();
        map.clear();
    });
}
