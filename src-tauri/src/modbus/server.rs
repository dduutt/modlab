use std::collections::HashMap;
use std::future::{ready, Ready};
use std::io;
use std::net::{Shutdown, TcpStream as StdTcpStream};
use std::sync::{Arc, Mutex, Weak};

use super::wire::{WireSink, WireTap};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_modbus::prelude::{Request, Response, SlaveRequest};
use tokio_modbus::server::tcp::Server as TcpServer;
use tokio_modbus::server::Service;

#[derive(Clone, Debug, Default)]
pub struct SessionMemory {
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    coils: Arc<Mutex<HashMap<u16, bool>>>,
    discrete_inputs: Arc<Mutex<HashMap<u16, bool>>>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self::default()
    }

    fn addresses(start: u16, count: u16) -> Result<impl Iterator<Item = u16>, String> {
        if count == 0 {
            return Err("Register count must be at least 1".into());
        }
        if u32::from(start) + u32::from(count) > 65_536 {
            return Err(format!(
                "Address range {start}..{} exceeds 65535",
                u32::from(start) + u32::from(count) - 1
            ));
        }
        Ok((0..count).map(move |offset| start + offset))
    }

    fn validate_count(count: u16, max: u16) -> Result<(), String> {
        if count > max {
            return Err(format!(
                "Requested count {count} exceeds the Modbus limit of {max}"
            ));
        }
        Ok(())
    }

    fn read_word_map(
        map: &Mutex<HashMap<u16, u16>>,
        start: u16,
        count: u16,
    ) -> Result<Vec<u16>, String> {
        Self::validate_count(count, 125)?;
        let map = map.lock().map_err(|_| "Session memory lock poisoned")?;
        Ok(Self::addresses(start, count)?
            .map(|addr| *map.get(&addr).unwrap_or(&0))
            .collect())
    }

    fn read_coil_map(
        map: &Mutex<HashMap<u16, bool>>,
        start: u16,
        count: u16,
    ) -> Result<Vec<bool>, String> {
        Self::validate_count(count, 2_000)?;
        let map = map.lock().map_err(|_| "Session memory lock poisoned")?;
        Ok(Self::addresses(start, count)?
            .map(|addr| *map.get(&addr).unwrap_or(&false))
            .collect())
    }

    fn set_word(map: &Mutex<HashMap<u16, u16>>, addr: u16, value: u16) {
        if let Ok(mut map) = map.lock() {
            map.insert(addr, value);
        }
    }

    fn set_coil(map: &Mutex<HashMap<u16, bool>>, addr: u16, value: bool) {
        if let Ok(mut map) = map.lock() {
            map.insert(addr, value);
        }
    }

    pub fn get_range(
        &self,
        function_code: &str,
        start: u16,
        count: u16,
    ) -> Result<HashMap<u16, u16>, String> {
        let values = match function_code {
            "0x01" => self
                .get_coil_range(start, count)?
                .into_iter()
                .map(u16::from)
                .collect(),
            "0x02" => self
                .get_discrete_input_range(start, count)?
                .into_iter()
                .map(u16::from)
                .collect(),
            "0x03" => self.get_holding_range(start, count)?,
            "0x04" => self.get_input_range(start, count)?,
            _ => return Err(format!("Unsupported function code: {function_code}")),
        };
        Ok(Self::addresses(start, count)?.zip(values).collect())
    }

    pub fn set_value(&self, function_code: &str, addr: u16, value: u16) -> Result<(), String> {
        match function_code {
            "0x01" => Self::set_coil(&self.coils, addr, value != 0),
            "0x02" => Self::set_coil(&self.discrete_inputs, addr, value != 0),
            "0x03" => Self::set_word(&self.holding_registers, addr, value),
            "0x04" => Self::set_word(&self.input_registers, addr, value),
            _ => return Err(format!("Unsupported function code: {function_code}")),
        }
        Ok(())
    }

    pub fn get_holding_range(&self, start: u16, count: u16) -> Result<Vec<u16>, String> {
        Self::read_word_map(&self.holding_registers, start, count)
    }

    pub fn get_input_range(&self, start: u16, count: u16) -> Result<Vec<u16>, String> {
        Self::read_word_map(&self.input_registers, start, count)
    }

    pub fn get_coil_range(&self, start: u16, count: u16) -> Result<Vec<bool>, String> {
        Self::read_coil_map(&self.coils, start, count)
    }

    pub fn get_discrete_input_range(&self, start: u16, count: u16) -> Result<Vec<bool>, String> {
        Self::read_coil_map(&self.discrete_inputs, start, count)
    }

    pub fn set_holding(&self, addr: u16, value: u16) {
        Self::set_word(&self.holding_registers, addr, value);
    }

    pub fn set_coil_value(&self, addr: u16, value: bool) {
        Self::set_coil(&self.coils, addr, value);
    }
}

#[derive(Default)]
struct TcpConnections {
    stopped: bool,
    sockets: Vec<Weak<StdTcpStream>>,
    peers: Vec<TcpPeer>,
    next_peer_id: u64,
    slaves: HashMap<u8, TcpSlaveRoute>,
    logger: Option<TcpLogSink>,
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcpClientInfo {
    pub id: u64,
    pub address: String,
    pub connected_at: u64,
    pub last_request_at: Option<u64>,
}
struct TcpPeer {
    socket: Weak<StdTcpStream>,
    info: TcpClientInfo,
}
fn timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcpTrafficLog {
    pub id: String,
    pub session_id: String,
    pub session_title: String,
    pub timestamp_ms: u64,
    pub direction: String,
    pub kind: String,
    pub level: String,
    pub detail: bool,
    pub protocol: String,
    pub complete: bool,
    pub message: String,
    pub bytes: String,
}
pub type TcpLogSink = Arc<dyn Fn(TcpTrafficLog) + Send + Sync>;

pub struct TcpSlaveRoute {
    pub session_id: String,
    pub session_title: String,
    pub memory: SessionMemory,
}

pub struct TcpSlaveTask {
    task: JoinHandle<()>,
    connections: Arc<Mutex<TcpConnections>>,
}

impl TcpSlaveTask {
    pub fn clients(&self) -> Vec<TcpClientInfo> {
        let mut state = self
            .connections
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        state.peers.retain(|peer| peer.socket.strong_count() > 0);
        state.peers.iter().map(|peer| peer.info.clone()).collect()
    }

    pub fn set_logger(&self, logger: Option<TcpLogSink>) {
        self.connections
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .logger = logger;
    }

    pub fn register(&self, unit_id: u8, route: TcpSlaveRoute) -> Result<(), String> {
        let mut state = self
            .connections
            .lock()
            .map_err(|_| "TCP route lock poisoned")?;
        if state.stopped || self.task.is_finished() {
            return Err("TCP listener is stopped".into());
        }
        if let Some(existing) = state.slaves.get(&unit_id) {
            return Err(format!(
                "Unit ID {unit_id} is already in use by session '{}'",
                existing.session_title
            ));
        }
        state.slaves.insert(unit_id, route);
        Ok(())
    }

    pub fn unregister(&self, unit_id: u8, session_id: &str) -> Result<usize, String> {
        // Use the same lock as request handling: once removed, even requests
        // on already accepted sockets can no longer access this session.
        let mut state = self
            .connections
            .lock()
            .map_err(|_| "TCP route lock poisoned")?;
        if state
            .slaves
            .get(&unit_id)
            .is_some_and(|route| route.session_id == session_id)
        {
            state.slaves.remove(&unit_id);
        }
        Ok(state.slaves.len())
    }

    fn close_connections(&self) {
        // Serialize shutdown with synchronous service calls so no request can
        // modify memory after stop has completed, even if already decoded.
        let mut connections = self
            .connections
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        connections.stopped = true;
        connections.slaves.clear();
        connections.peers.clear();
        for socket in connections
            .sockets
            .drain(..)
            .filter_map(|socket| socket.upgrade())
        {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }

    pub async fn stop(mut self) {
        self.close_connections();
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for TcpSlaveTask {
    fn drop(&mut self) {
        self.close_connections();
        self.task.abort();
    }
}

pub fn start_tcp_slave(listener: TcpListener) -> TcpSlaveTask {
    let connections = Arc::new(Mutex::new(TcpConnections::default()));
    let server_connections = connections.clone();
    let task = tokio::spawn(async move {
        let server = TcpServer::new(listener);
        let on_connected = move |stream: TcpStream, peer: std::net::SocketAddr| {
            let connections = server_connections.clone();
            async move {
                let stream = stream.into_std()?;
                let socket = Arc::new(stream.try_clone()?);
                let stream = TcpStream::from_std(stream)?;
                let client_id = {
                    let mut state = connections.lock().map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "TCP connection lock poisoned")
                    })?;
                    if state.stopped {
                        let _ = socket.shutdown(Shutdown::Both);
                        return Ok(None);
                    }
                    state.sockets.retain(|socket| socket.strong_count() > 0);
                    state.sockets.push(Arc::downgrade(&socket));
                    state.next_peer_id += 1;
                    let id = state.next_peer_id;
                    state.peers.retain(|peer| peer.socket.strong_count() > 0);
                    state.peers.push(TcpPeer {
                        socket: Arc::downgrade(&socket),
                        info: TcpClientInfo {
                            id,
                            address: peer.to_string(),
                            connected_at: timestamp_ms(),
                            last_request_at: None,
                        },
                    });
                    id
                };
                let log_connections = connections.clone();
                let sink: WireSink = Arc::new(move |direction, bytes, complete| {
                    use std::sync::atomic::{AtomicU64, Ordering};
                    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
                    let (logger, routes) = {
                        let state = log_connections
                            .lock()
                            .unwrap_or_else(|err| err.into_inner());
                        let matched = bytes.get(6).and_then(|id| state.slaves.get(id));
                        let routes: Vec<_> = if let Some(route) = matched {
                            vec![(route.session_id.clone(), route.session_title.clone())]
                        } else {
                            state
                                .slaves
                                .values()
                                .map(|route| {
                                    (route.session_id.clone(), route.session_title.clone())
                                })
                                .collect()
                        };
                        (state.logger.clone(), routes)
                    };
                    if let Some(logger) = logger {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        for (session_id, session_title) in routes {
                            logger(TcpTrafficLog {
                                id: format!(
                                    "tcp-wire-{now}-{}",
                                    SEQUENCE.fetch_add(1, Ordering::Relaxed)
                                ),
                                session_id,
                                session_title,
                                timestamp_ms: now,
                                direction: direction.into(),
                                kind: "frame".into(),
                                level:
                                    if !complete || bytes.get(7).is_some_and(|fc| fc & 0x80 != 0) {
                                        "error"
                                    } else {
                                        "info"
                                    }
                                    .into(),
                                detail: false,
                                protocol: "TCP".into(),
                                complete,
                                message: if !complete {
                                    "Incomplete / unframed data"
                                } else {
                                    ""
                                }
                                .into(),
                                bytes: bytes
                                    .iter()
                                    .map(|b| format!("{b:02X}"))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            });
                        }
                    }
                });
                let stream = WireTap::new(stream, sink, true, false);
                let service = TcpSlaveService {
                    client_id,
                    connections,
                    _socket: socket,
                };
                Ok(Some((service, stream)))
            }
        };
        if let Err(err) = server
            .serve(&on_connected, |err| {
                eprintln!("Modbus TCP client error: {err}");
            })
            .await
        {
            eprintln!("Modbus TCP server stopped: {err}");
        }
    });
    TcpSlaveTask { task, connections }
}

struct TcpSlaveService {
    client_id: u64,
    connections: Arc<Mutex<TcpConnections>>,
    // Keep the shutdown handle alive exactly as long as this client service.
    _socket: Arc<StdTcpStream>,
}

impl Service for TcpSlaveService {
    type Request = SlaveRequest<'static>;
    type Response = Response;
    type Error = io::Error;
    type Future = Ready<Result<Response, io::Error>>;

    fn call(&self, request: SlaveRequest<'static>) -> Self::Future {
        use Request::*;

        let mut connections = self
            .connections
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        if connections.stopped {
            return ready(Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "TCP slave stopped",
            )));
        }
        if let Some(peer) = connections
            .peers
            .iter_mut()
            .find(|peer| peer.info.id == self.client_id)
        {
            peer.info.last_request_at = Some(timestamp_ms());
        }
        let function_code = request_function_code(&request.request);
        let Some(route) = connections.slaves.get(&request.slave) else {
            // Gateway target failed to respond. Keep the TCP connection open
            // so its next request can address another registered Unit ID.
            return ready(Ok(exception_response(function_code, 0x0B)));
        };
        let memory = &route.memory;
        let response = match request.request {
            ReadCoils(start, count) => memory.get_coil_range(start, count).map(Response::ReadCoils),
            ReadDiscreteInputs(start, count) => memory
                .get_discrete_input_range(start, count)
                .map(Response::ReadDiscreteInputs),
            ReadHoldingRegisters(start, count) => memory
                .get_holding_range(start, count)
                .map(Response::ReadHoldingRegisters),
            ReadInputRegisters(start, count) => memory
                .get_input_range(start, count)
                .map(Response::ReadInputRegisters),
            WriteSingleCoil(addr, value) => {
                memory.set_coil_value(addr, value);
                Ok(Response::WriteSingleCoil(addr, value))
            }
            WriteSingleRegister(addr, value) => {
                memory.set_holding(addr, value);
                Ok(Response::WriteSingleRegister(addr, value))
            }
            WriteMultipleCoils(start, values) => {
                for (offset, value) in values.iter().copied().enumerate() {
                    let addr = start
                        .checked_add(offset as u16)
                        .ok_or_else(|| "Coil address range exceeds 65535".to_string());
                    if let Ok(addr) = addr {
                        memory.set_coil_value(addr, value);
                    } else {
                        return ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Coil address range exceeds 65535",
                        )));
                    }
                }
                Ok(Response::WriteMultipleCoils(start, values.len() as u16))
            }
            WriteMultipleRegisters(start, values) => {
                for (offset, value) in values.iter().copied().enumerate() {
                    let addr = start
                        .checked_add(offset as u16)
                        .ok_or_else(|| "Register address range exceeds 65535".to_string());
                    if let Ok(addr) = addr {
                        memory.set_holding(addr, value);
                    } else {
                        return ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Register address range exceeds 65535",
                        )));
                    }
                }
                Ok(Response::WriteMultipleRegisters(start, values.len() as u16))
            }
            _ => Err("Unsupported Modbus request".into()),
        };

        ready(response.map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message)))
    }
}

fn request_function_code(request: &Request<'_>) -> u8 {
    use Request::*;
    match request {
        ReadCoils(..) => 0x01,
        ReadDiscreteInputs(..) => 0x02,
        ReadHoldingRegisters(..) => 0x03,
        ReadInputRegisters(..) => 0x04,
        WriteSingleCoil(..) => 0x05,
        WriteSingleRegister(..) => 0x06,
        WriteMultipleCoils(..) => 0x0F,
        WriteMultipleRegisters(..) => 0x10,
        MaskWriteRegister(..) => 0x16,
        ReadWriteMultipleRegisters(..) => 0x17,
        Custom(code, _) => *code,
        Disconnect => 0,
    }
}

fn exception_response(function_code: u8, exception: u8) -> Response {
    Response::Custom(function_code | 0x80, vec![exception].into())
}
