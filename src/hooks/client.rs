use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
use tokio::sync::{mpsc, oneshot, Mutex as AsyncMutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const BEACON_MAGIC: &[u8; 4] = b"OZN\x01";
// sent to a server directly when broadcast discovery can't reach it
const QUERY_MAGIC: &[u8; 4] = b"OZN?";
// A server that hasn't broadcast for this long is considered gone.
const BEACON_STALE_AFTER: Duration = Duration::from_secs(5);
const BEACON_RETRY_DELAY: Duration = Duration::from_secs(3);
const BEACON_RECV_TIMEOUT: Duration = Duration::from_secs(1);
// Ping the server periodically; if nothing at all arrives for the timeout
// (a silent network drop sends no FIN/RST, so the socket read alone would
// block forever), declare the connection dead.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Clone, PartialEq)]
pub struct ClientConfig {
    pub api_version: ApiVersion,
    pub codec: CodecId,
    pub compression: CompressionId,
    pub encryption: EncryptionId,
    pub beacon_port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_version: ApiVersion::new(0, 1, 0),
            codec: CodecId::Json,
            compression: CompressionId::None,
            encryption: EncryptionId::None,
            beacon_port: 18051,
        }
    }
}

#[derive(Clone)]
pub struct ServerInfo {
    pub host: String,
    pub port: u16,
    pub api_version: ApiVersion,
}

type ReplyTx = oneshot::Sender<Result<Frame, String>>;

pub struct Client {
    codec: Arc<JsonCodec>,
    sink: Arc<AsyncMutex<WsSink>>,
    pending: Arc<Mutex<HashMap<u32, ReplyTx>>>,
    next_call_id: AtomicU32,
    info: ServerInfo,
    disconnect: AsyncMutex<Option<oneshot::Receiver<String>>>,
}

impl Client {
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    /// Close the WebSocket so the server doesn't keep a zombie session
    /// around after a manual disconnect.
    pub async fn close(&self) {
        let _ = self.sink.lock().await.close().await;
    }

    pub async fn wait_disconnect(&self) -> String {
        let rx = self.disconnect.lock().await.take();
        match rx {
            Some(rx) => rx
                .await
                .unwrap_or_else(|_| "Connection closed.".to_string()),
            None => std::future::pending().await,
        }
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
                Err(format!("Server reported an error [{module}-{code:04}]: {detail}"))
            }
            _ => Err("Unexpected frame from server".into()),
        }
    }

    async fn call_raw(&self, cmd: CommandId, payload: Vec<u8>) -> Result<Frame, String> {
        let call_id = self.next_call_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel::<Result<Frame, String>>();
        self.pending.lock().unwrap().insert(call_id, tx);

        let frame = Frame::request(call_id, cmd.namespace, cmd.command, payload);
        let bytes = self.codec.encode(&frame).map_err(|e| format!("Encode: {e}"))?;

        {
            let mut sink = self.sink.lock().await;
            if let Err(e) = sink.send(Message::Binary(bytes)).await {
                self.pending.lock().unwrap().remove(&call_id);
                return Err(format!("Failed to send request: {e} (the connection looks dead)."));
            }
        }

        match rx.await {
            Ok(result) => result,
            Err(_) => Err("Connection closed before the server replied.".into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiscoveredServer {
    pub host: String,
    pub port: u16,
}

/// Bind with SO_REUSEADDR/SO_REUSEPORT so several app instances (or an old
/// listener thread that hasn't wound down yet) can all hear the beacons —
/// broadcast datagrams are delivered to every reuse-bound socket.
fn bind_beacon_socket(port: u16) -> std::io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&std::net::SocketAddr::from(([0, 0, 0, 0], port)).into())?;
    Ok(socket.into())
}

/// Listen for server beacons for as long as the returned receiver is alive.
/// Each `Ok` update carries the full set of servers currently broadcasting;
/// an `Err` reports a listener problem (e.g. the beacon port is taken), after
/// which listening is retried automatically.
pub fn watch_beacons(config: &ClientConfig) -> mpsc::UnboundedReceiver<Result<Vec<DiscoveredServer>, String>> {
    let beacon_port = config.beacon_port;
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || {
        'bind: while !tx.is_closed() {
            let socket = match bind_beacon_socket(beacon_port) {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to bind beacon port {beacon_port}: {e}")));
                    std::thread::sleep(BEACON_RETRY_DELAY);
                    continue 'bind;
                }
            };
            if let Err(e) = socket.set_read_timeout(Some(BEACON_RECV_TIMEOUT)) {
                let _ = tx.send(Err(format!("Failed to set beacon timeout: {e}")));
                std::thread::sleep(BEACON_RETRY_DELAY);
                continue 'bind;
            }

            let mut seen: HashMap<DiscoveredServer, Instant> = HashMap::new();
            // Force a snapshot on the first pass so a previously reported
            // error is cleared once listening recovers.
            let mut last_sent: Option<Vec<DiscoveredServer>> = None;
            let mut buf = [0u8; 64];
            while !tx.is_closed() {
                match socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        if len >= 6 && &buf[..4] == BEACON_MAGIC {
                            let port = u16::from_le_bytes([buf[4], buf[5]]);
                            let server = DiscoveredServer { host: src.ip().to_string(), port };
                            if seen.insert(server, Instant::now()).is_none() {
                                log::info!("Beacon from {src}: server port = {port}");
                            }
                        }
                    }
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(e) => {
                        let _ = tx.send(Err(format!("Beacon listener error: {e}")));
                        std::thread::sleep(BEACON_RETRY_DELAY);
                        continue 'bind;
                    }
                }

                seen.retain(|_, last| last.elapsed() < BEACON_STALE_AFTER);
                let mut current: Vec<DiscoveredServer> = seen.keys().cloned().collect();
                current.sort();
                if last_sent.as_ref() != Some(&current) {
                    if tx.send(Ok(current.clone())).is_err() {
                        return;
                    }
                    last_sent = Some(current);
                }
            }
            return;
        }
    });

    rx
}

/// Ask one host directly for its current TCP port. Broadcast discovery dies on
/// Wi-Fi with client isolation, but a unicast query/reply still gets through, so
/// this backs the "connect by IP" path. Sends QUERY_MAGIC to host:beacon_port and
/// waits for the same magic+port reply the beacon uses. Blocking, run it off the
/// async runtime (spawn_blocking).
pub fn query_server_port(host: &str, beacon_port: u16, timeout: Duration) -> Result<u16, String> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| format!("bind failed: {e}"))?;
    // short per-recv timeout so we can resend, UDP over Wi-Fi can drop a packet
    socket
        .set_read_timeout(Some(Duration::from_millis(400)))
        .map_err(|e| format!("set timeout failed: {e}"))?;

    let deadline = Instant::now() + timeout;
    let mut buf = [0u8; 64];
    while Instant::now() < deadline {
        socket
            .send_to(QUERY_MAGIC, (host, beacon_port))
            .map_err(|e| format!("send failed: {e}"))?;
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                // ignore stray packets, only take a real reply from the host we asked
                if len >= 6 && &buf[..4] == BEACON_MAGIC && src.ip().to_string() == host {
                    return Ok(u16::from_le_bytes([buf[4], buf[5]]));
                }
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(e) => return Err(format!("recv failed: {e}")),
        }
    }
    Err("No response (is the game running and the IP correct?)".into())
}

pub async fn connect(host: &str, port: u16, config: &ClientConfig) -> Result<Arc<Client>, String> {
    let url = format!("ws://{host}:{port}");
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {e}"))?;

    // turn off Nagle so small request/reply frames don't sit waiting on an ack
    if let MaybeTlsStream::Plain(tcp) = ws.get_ref() {
        let _ = tcp.set_nodelay(true);
    }

    let (sink, mut stream) = ws.split();
    let sink = Arc::new(AsyncMutex::new(sink));

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
    let pending: Arc<Mutex<HashMap<u32, ReplyTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (disc_tx, disc_rx) = oneshot::channel::<String>();

    spawn_reader(stream, sink.clone(), codec.clone(), pending.clone(), disc_tx);

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
        disconnect: AsyncMutex::new(Some(disc_rx)),
    }))
}

fn spawn_reader(
    mut stream: WsStream,
    sink: Arc<AsyncMutex<WsSink>>,
    codec: Arc<JsonCodec>,
    pending: Arc<Mutex<HashMap<u32, ReplyTx>>>,
    disconnect: oneshot::Sender<String>,
) {
    tokio::spawn(async move {
        let mut last_rx = Instant::now();
        let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let reason = loop {
            tokio::select! {
                msg = stream.next() => match msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        last_rx = Instant::now();
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
                            let _ = tx.send(Ok(frame));
                        }
                    }
                    Some(Ok(Message::Close(_))) => break "The server closed the connection.".to_string(),
                    Some(Ok(_)) => last_rx = Instant::now(),
                    Some(Err(e)) => {
                        break format!("Connection error: {e}. The server (the game) likely crashed.")
                    }
                    None => {
                        break "Connection dropped with no clean close - the server (the game) likely crashed."
                            .to_string()
                    }
                },
                _ = heartbeat.tick() => {
                    if last_rx.elapsed() > HEARTBEAT_TIMEOUT {
                        break format!(
                            "No response from the server for {}s - the network or the game went away.",
                            HEARTBEAT_TIMEOUT.as_secs()
                        );
                    }
                    if sink.lock().await.send(Message::Ping(Vec::new())).await.is_err() {
                        break "Connection lost (failed to send heartbeat).".to_string();
                    }
                }
            }
        };

        {
            let mut map = pending.lock().unwrap();
            for (_call_id, tx) in map.drain() {
                let _ = tx.send(Err(reason.clone()));
            }
        }
        let _ = disconnect.send(reason);
    });
}
