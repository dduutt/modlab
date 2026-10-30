use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tokio_modbus::client::Context as ModbusContext;
use tokio_modbus::prelude::*;
use tokio_modbus::server::tcp::Server as TcpServer;
use tokio_modbus::server::Service;
use tokio_modbus::ExceptionCode;
use tokio_serial::SerialPortBuilderExt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsmState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub protocol: String, // "TCP" | "RTU"
    pub ip: String,
    pub port: u16,
    pub serial_port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub role: String, // "Master" | "Slave"
    pub unit_id: u8,
    pub function_code: String,
    pub start_address: u16,
    pub count: u16,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritePayload {
    pub address: u16,
    pub value: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataGenConfig {
    pub start_address: u16,
    pub count: u16,
    pub function_code: String,
    pub data_type: String,
    pub byte_order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SessionCommand {
    Connect(ConnectionConfig),
    Disconnect,
    UpdateConfig(ConnectionConfig),
    StartPolling(u64), // interval in ms
    StopPolling,
    ReadOnce,
    WriteRegister(WritePayload),
    WriteRegistersBatch(HashMap<u16, u16>),
    StartAutoIncrement {
        #[serde(rename = "intervalMs")]
        interval_ms: u64,
        config: DataGenConfig,
    },
    StopAutoIncrement,
    FillRandom(DataGenConfig),
}

#[derive(Debug)]
pub struct CommandEnvelope {
    pub cmd: SessionCommand,
    pub resp: Option<tokio::sync::oneshot::Sender<Result<(), String>>>,
}

impl From<SessionCommand> for CommandEnvelope {
    fn from(cmd: SessionCommand) -> Self {
        Self { cmd, resp: None }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum SessionEvent {
    StateChanged(FsmState),
    DataUpdated(HashMap<u16, u16>),
    PollError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpClientInfo {
    pub id: usize,
    pub address: String,
    pub connected_at: u64,
    pub last_request_at: Option<u64>,
}

pub struct ClientDropGuard {
    pub client_id: usize,
    pub clients: Arc<RwLock<HashMap<usize, TcpClientInfo>>>,
}

impl Drop for ClientDropGuard {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.clients.write() {
            lock.remove(&self.client_id);
        }
    }
}

#[inline]
fn check_bounds(start: u16, count: u16, max_count: u16) -> Result<(), ExceptionCode> {
    if count == 0 || count > max_count {
        return Err(ExceptionCode::IllegalDataValue);
    }
    if (start as u32) + (count as u32) > 65536 {
        return Err(ExceptionCode::IllegalDataAddress);
    }
    Ok(())
}

#[derive(Clone)]
struct SlaveService {
    client_id: usize,
    _unit_id: u8,
    memory: Arc<RwLock<HashMap<u16, u16>>>,
    event_sender: mpsc::Sender<SessionEvent>,
    clients: Arc<RwLock<HashMap<usize, TcpClientInfo>>>,
    _guard: Arc<ClientDropGuard>,
}

impl Service for SlaveService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if let Ok(mut lock) = self.clients.write() {
            if let Some(info) = lock.get_mut(&self.client_id) {
                info.last_request_at = Some(now);
            }
        }

        let res = match req {
            Request::ReadHoldingRegisters(start, count) => {
                check_bounds(start, count, 125).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<u16> = (0..count).map(|i| *mem.get(&(start + i)).unwrap_or(&0)).collect();
                    Response::ReadHoldingRegisters(values)
                })
            }
            Request::ReadInputRegisters(start, count) => {
                check_bounds(start, count, 125).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<u16> = (0..count).map(|i| *mem.get(&(start + i)).unwrap_or(&0)).collect();
                    Response::ReadInputRegisters(values)
                })
            }
            Request::WriteSingleRegister(addr, val) => {
                {
                    let mut mem = self.memory.write().unwrap();
                    mem.insert(addr, val);
                }
                let mut update_map = HashMap::new();
                update_map.insert(addr, val);
                let _ = self.event_sender.try_send(SessionEvent::DataUpdated(update_map));
                Ok(Response::WriteSingleRegister(addr, val))
            }
            Request::WriteMultipleRegisters(start, vals) => {
                let count = vals.len() as u16;
                match check_bounds(start, count, 123) {
                    Ok(_) => {
                        let mut update_map = HashMap::new();
                        {
                            let mut mem = self.memory.write().unwrap();
                            for (i, val) in vals.iter().copied().enumerate() {
                                let addr = start + i as u16;
                                mem.insert(addr, val);
                                update_map.insert(addr, val);
                            }
                        }
                        let _ = self.event_sender.try_send(SessionEvent::DataUpdated(update_map));
                        Ok(Response::WriteMultipleRegisters(start, count))
                    }
                    Err(e) => Err(e),
                }
            }
            Request::ReadCoils(start, count) => {
                check_bounds(start, count, 2000).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<bool> = (0..count).map(|i| mem.get(&(start + i)).copied().unwrap_or(0) != 0).collect();
                    Response::ReadCoils(values)
                })
            }
            Request::ReadDiscreteInputs(start, count) => {
                check_bounds(start, count, 2000).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<bool> = (0..count).map(|i| mem.get(&(start + i)).copied().unwrap_or(0) != 0).collect();
                    Response::ReadDiscreteInputs(values)
                })
            }
            Request::WriteSingleCoil(addr, val) => {
                let int_val = if val { 1 } else { 0 };
                {
                    let mut mem = self.memory.write().unwrap();
                    mem.insert(addr, int_val);
                }
                let mut update_map = HashMap::new();
                update_map.insert(addr, int_val);
                let _ = self.event_sender.try_send(SessionEvent::DataUpdated(update_map));
                Ok(Response::WriteSingleCoil(addr, val))
            }
            _ => Err(ExceptionCode::IllegalFunction),
        };

        ready(res)
    }
}

/// Actor: 独立协程内部状态，独占通信资源，完全无锁竞争
pub struct SessionActor {
    _id: String,
    state: FsmState,
    receiver: mpsc::Receiver<CommandEnvelope>,
    event_sender: mpsc::Sender<SessionEvent>,
    config: Option<ConnectionConfig>,
    client: Option<ModbusContext>,
    slave_stop: Option<tokio::sync::oneshot::Sender<()>>,
    memory: Arc<RwLock<HashMap<u16, u16>>>,
    clients: Arc<RwLock<HashMap<usize, TcpClientInfo>>>,
    is_polling: bool,
    poll_interval: u64,
    is_auto_incrementing: bool,
    auto_inc_interval: u64,
    auto_inc_config: Option<DataGenConfig>,
}

impl SessionActor {
    pub fn new(
        id: String,
        receiver: mpsc::Receiver<CommandEnvelope>,
        event_sender: mpsc::Sender<SessionEvent>,
        clients: Arc<RwLock<HashMap<usize, TcpClientInfo>>>,
    ) -> Self {
        Self {
            _id: id,
            state: FsmState::Disconnected,
            receiver,
            event_sender,
            config: None,
            client: None,
            slave_stop: None,
            memory: Arc::new(RwLock::new(HashMap::new())),
            clients,
            is_polling: false,
            poll_interval: 1000,
            is_auto_incrementing: false,
            auto_inc_interval: 1000,
            auto_inc_config: None,
        }
    }

    async fn set_state(&mut self, new_state: FsmState) {
        if self.state != new_state {
            self.state = new_state.clone();
            let _ = self
                .event_sender
                .send(SessionEvent::StateChanged(new_state))
                .await;
        }
    }

    pub async fn run(mut self) {
        let mut poll_interval = time::interval(Duration::from_millis(self.poll_interval));
        poll_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        let mut auto_inc_interval = time::interval(Duration::from_millis(self.auto_inc_interval.max(100)));
        auto_inc_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                envelope_opt = self.receiver.recv() => {
                    match envelope_opt {
                        Some(envelope) => self.handle_command(envelope, &mut poll_interval, &mut auto_inc_interval).await,
                        None => break, // Channel dropped by frontend, exit cleanly
                    }
                }
                _ = poll_interval.tick(), if self.is_polling && self.state == FsmState::Connected => {
                    self.execute_poll().await;
                }
                _ = auto_inc_interval.tick(), if self.is_auto_incrementing => {
                    self.execute_auto_increment().await;
                }
            }
        }
        self.cleanup().await;
        let _ = self.set_state(FsmState::Disconnected).await;
    }

    async fn cleanup(&mut self) {
        self.is_auto_incrementing = false;
        if let Some(stop) = self.slave_stop.take() {
            let _ = stop.send(());
        }
        if let Some(mut ctx) = self.client.take() {
            let _ = ctx.disconnect().await;
        }
        if let Ok(mut lock) = self.clients.write() {
            lock.clear();
        }
    }

    async fn emit_error(&mut self, msg: impl Into<String>) {
        let _ = self.event_sender.send(SessionEvent::PollError(msg.into())).await;
    }

    async fn emit_data(&mut self, map: HashMap<u16, u16>) {
        if !map.is_empty() {
            let _ = self.event_sender.send(SessionEvent::DataUpdated(map)).await;
        }
    }

    fn role_is(&self, role: &str) -> bool {
        self.config.as_ref().map(|c| c.role == role).unwrap_or(false)
    }

    /// 轮询/自增运行期间禁止单次读取、写入、随机填充（互斥约定）
    fn ensure_idle(&self, action: &str) -> Result<(), String> {
        if self.is_polling {
            return Err(format!("Cannot {} while polling is active", action));
        }
        if self.is_auto_incrementing {
            return Err(format!("Cannot {} while auto-increment is active", action));
        }
        Ok(())
    }

    fn ensure_writable_fc(fc: &str) -> Result<(), String> {
        if is_readonly_function(fc) {
            return Err(format!("Function code {} is read-only", fc));
        }
        Ok(())
    }

    /// 集中校验：所有命令的前置约束只在这里声明一次
    fn guard(&self, cmd: &SessionCommand) -> Result<(), String> {
        let connected = self.state == FsmState::Connected;
        match cmd {
            SessionCommand::StartPolling(_) => {
                if self.role_is("Slave") {
                    return Err("Polling is only supported in Master mode".into());
                }
                if !connected {
                    return Err("Cannot poll while disconnected".into());
                }
            }
            SessionCommand::ReadOnce => {
                if self.role_is("Slave") {
                    return Err("Read once is only supported in Master mode".into());
                }
                self.ensure_idle("read once")?;
                if !connected {
                    return Err("Cannot read while disconnected".into());
                }
            }
            SessionCommand::WriteRegister(_) | SessionCommand::WriteRegistersBatch(_) => {
                self.ensure_idle("write")?;
                if let Some(cfg) = &self.config {
                    Self::ensure_writable_fc(&cfg.function_code)?;
                }
            }
            SessionCommand::StartAutoIncrement { config, .. } => {
                if self.role_is("Master") {
                    return Err("Auto-increment is only supported in Slave mode".into());
                }
                if !connected {
                    return Err("Cannot start auto-increment while disconnected".into());
                }
                Self::ensure_writable_fc(&config.function_code)?;
            }
            SessionCommand::FillRandom(config) => {
                self.ensure_idle("fill random")?;
                if self.role_is("Master") {
                    return Err("Random fill is only supported in Slave mode".into());
                }
                Self::ensure_writable_fc(&config.function_code)?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command(
        &mut self,
        envelope: CommandEnvelope,
        interval: &mut time::Interval,
        auto_inc_interval: &mut time::Interval,
    ) {
        let CommandEnvelope { cmd, resp } = envelope;
        let respond = |res: Result<(), String>| {
            if let Some(r) = resp {
                let _ = r.send(res);
            }
        };

        if let Err(msg) = self.guard(&cmd) {
            self.emit_error(msg.clone()).await;
            respond(Err(msg));
            return;
        }

        let res = match cmd {
            SessionCommand::Connect(config) => {
                self.is_polling = false;
                self.cleanup().await;
                self.config = Some(config.clone());
                self.set_state(FsmState::Connecting).await;

                match (config.role.as_str(), config.protocol.as_str()) {
                    ("Slave", "TCP") => self.connect_slave_tcp(&config).await,
                    ("Slave", _) => {
                        self.set_state(FsmState::Error("RTU Slave not supported on this platform".into())).await;
                    }
                    (_, "TCP") => self.connect_master_tcp(&config).await,
                    _ => self.connect_master_rtu(&config).await,
                }
                Ok(())
            }
            SessionCommand::Disconnect => {
                self.is_polling = false;
                self.cleanup().await;
                self.set_state(FsmState::Disconnected).await;
                Ok(())
            }
            SessionCommand::UpdateConfig(config) => {
                if let Some(cfg) = &mut self.config {
                    cfg.unit_id = config.unit_id;
                    cfg.function_code = config.function_code;
                    cfg.start_address = config.start_address;
                    cfg.count = config.count;
                    if let Some(t) = config.timeout_ms {
                        cfg.timeout_ms = Some(t);
                    }
                } else {
                    self.config = Some(config);
                }
                Ok(())
            }
            SessionCommand::StartPolling(interval_ms) => {
                let min_interval = if self.config.as_ref().map(|c| c.protocol == "RTU").unwrap_or(false) {
                    100
                } else {
                    20
                };
                let safe_interval = interval_ms.max(min_interval);
                self.poll_interval = safe_interval;
                self.is_polling = true;
                *interval = time::interval(Duration::from_millis(safe_interval));
                interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                Ok(())
            }
            SessionCommand::StopPolling => {
                self.is_polling = false;
                Ok(())
            }
            SessionCommand::ReadOnce => {
                self.execute_poll().await;
                Ok(())
            }
            SessionCommand::WriteRegister(payload) => self.write_register(payload).await,
            SessionCommand::WriteRegistersBatch(batch) => self.write_registers_batch(batch).await,
            SessionCommand::StartAutoIncrement { interval_ms, config } => {
                let safe_interval = interval_ms.max(100);
                self.auto_inc_interval = safe_interval;
                self.auto_inc_config = Some(config);
                self.is_auto_incrementing = true;
                *auto_inc_interval = time::interval(Duration::from_millis(safe_interval));
                auto_inc_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                auto_inc_interval.tick().await;
                Ok(())
            }
            SessionCommand::StopAutoIncrement => {
                self.is_auto_incrementing = false;
                self.auto_inc_config = None;
                Ok(())
            }
            SessionCommand::FillRandom(config) => self.execute_fill_random(&config).await,
        };
        respond(res);
    }

    async fn connect_slave_tcp(&mut self, config: &ConnectionConfig) {
        let bind_addr = format!("{}:{}", config.ip, config.port);
        let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
            Ok(l) => l,
            Err(e) => {
                self.set_state(FsmState::Error(format!("Bind {} failed: {}", bind_addr, e))).await;
                return;
            }
        };

        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        self.slave_stop = Some(stop_tx);
        self.set_state(FsmState::Connected).await;

        let memory = self.memory.clone();
        let event_sender = self.event_sender.clone();
        let unit_id = config.unit_id;
        let clients = self.clients.clone();
        let next_client_id = Arc::new(AtomicUsize::new(1));

        tokio::spawn(async move {
            let server = TcpServer::new(listener);
            let on_connected = move |stream: tokio::net::TcpStream, peer: SocketAddr| {
                let client_id = next_client_id.fetch_add(1, Ordering::Relaxed);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if let Ok(mut lock) = clients.write() {
                    lock.insert(
                        client_id,
                        TcpClientInfo {
                            id: client_id,
                            address: peer.to_string(),
                            connected_at: now,
                            last_request_at: None,
                        },
                    );
                }
                let guard = Arc::new(ClientDropGuard {
                    client_id,
                    clients: clients.clone(),
                });
                let service = SlaveService {
                    client_id,
                    _unit_id: unit_id,
                    memory: memory.clone(),
                    event_sender: event_sender.clone(),
                    clients: clients.clone(),
                    _guard: guard,
                };
                ready(Ok(Some((service, stream))))
            };
            let _ = server
                .serve_until(&on_connected, |_err| {}, async move {
                    let _ = stop_rx.await;
                })
                .await;
        });
    }

    async fn connect_master_tcp(&mut self, config: &ConnectionConfig) {
        let addr_str = format!("{}:{}", config.ip, config.port);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(a) => a,
            Err(_) => {
                self.set_state(FsmState::Error(format!("Invalid IP:Port '{}'", addr_str))).await;
                return;
            }
        };
        let timeout_dur = Duration::from_millis(config.timeout_ms.unwrap_or(1000).clamp(500, 10000));
        match time::timeout(timeout_dur, tcp::connect(addr)).await {
            Ok(Ok(mut ctx)) => {
                ctx.set_slave(Slave(config.unit_id));
                self.client = Some(ctx);
                self.set_state(FsmState::Connected).await;
            }
            Ok(Err(e)) => {
                self.set_state(FsmState::Error(format!("TCP Connect failed: {}", e))).await;
            }
            Err(_) => {
                self.set_state(FsmState::Error(format!("Connect timeout ({}ms)", timeout_dur.as_millis()))).await;
            }
        }
    }

    async fn connect_master_rtu(&mut self, config: &ConnectionConfig) {
        let builder = tokio_serial::new(&config.serial_port, config.baud_rate)
            .data_bits(match config.data_bits {
                7 => tokio_serial::DataBits::Seven,
                _ => tokio_serial::DataBits::Eight,
            })
            .stop_bits(match config.stop_bits {
                2 => tokio_serial::StopBits::Two,
                _ => tokio_serial::StopBits::One,
            })
            .parity(match config.parity.as_str() {
                "Even" => tokio_serial::Parity::Even,
                "Odd" => tokio_serial::Parity::Odd,
                _ => tokio_serial::Parity::None,
            });
        match builder.open_native_async() {
            Ok(port) => {
                self.client = Some(tokio_modbus::client::rtu::attach_slave(port, Slave(config.unit_id)));
                self.set_state(FsmState::Connected).await;
            }
            Err(e) => {
                self.set_state(FsmState::Error(format!("Open serial {} failed: {}", config.serial_port, e))).await;
            }
        }
    }

    /// Slave 本地写内存并通知前端
    async fn write_local(&mut self, map: HashMap<u16, u16>) {
        {
            let mut mem = self.memory.write().unwrap();
            mem.extend(map.iter().map(|(a, v)| (*a, *v)));
        }
        self.emit_data(map).await;
    }

    async fn write_register(&mut self, payload: WritePayload) -> Result<(), String> {
        if self.role_is("Slave") {
            self.write_local(HashMap::from([(payload.address, payload.value)])).await;
            return Ok(());
        }
        let is_coil = self.config.as_ref().map(|c| is_coil_function(&c.function_code)).unwrap_or(false);
        let ctx = self.client.as_mut().ok_or_else(|| "Not connected".to_string())?;
        let res = if is_coil {
            ctx.write_single_coil(payload.address, payload.value != 0).await
        } else {
            ctx.write_single_register(payload.address, payload.value).await
        };
        match res {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(ex)) => {
                let msg = format!("Write exception: {:?}", ex);
                self.emit_error(&msg).await;
                Err(msg)
            }
            Err(e) => {
                let msg = format!("Write failed: {}", e);
                self.emit_error(&msg).await;
                Err(msg)
            }
        }
    }

    async fn write_registers_batch(&mut self, batch: HashMap<u16, u16>) -> Result<(), String> {
        if self.role_is("Slave") {
            self.write_local(batch).await;
            return Ok(());
        }
        let is_coil = self.config.as_ref().map(|c| is_coil_function(&c.function_code)).unwrap_or(false);
        let ctx = self.client.as_mut().ok_or_else(|| "Not connected".to_string())?;

        if is_coil {
            for (addr, val) in &batch {
                let res = ctx.write_single_coil(*addr, *val != 0).await;
                match res {
                    Ok(Ok(_)) => {}
                    Ok(Err(ex)) => {
                        let msg = format!("Write coil exception: {:?}", ex);
                        self.emit_error(&msg).await;
                        return Err(msg);
                    }
                    Err(e) => {
                        let msg = format!("Write coil failed: {}", e);
                        self.emit_error(&msg).await;
                        return Err(msg);
                    }
                }
            }
            return Ok(());
        }
        let mut addrs: Vec<u16> = batch.keys().copied().collect();
        addrs.sort_unstable();
        let Some(&start) = addrs.first() else { return Ok(()); };
        let count = addrs.len() as u16;
        let is_contiguous = addrs.last().copied() == Some(start + count - 1);
        if is_contiguous && count <= 123 {
            let vals: Vec<u16> = addrs.iter().map(|a| batch[a]).collect();
            let res = ctx.write_multiple_registers(start, &vals).await;
            match res {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(ex)) => {
                    let msg = format!("Write multiple registers exception: {:?}", ex);
                    self.emit_error(&msg).await;
                    Err(msg)
                }
                Err(e) => {
                    let msg = format!("Write multiple registers failed: {}", e);
                    self.emit_error(&msg).await;
                    Err(msg)
                }
            }
        } else {
            for (addr, val) in &batch {
                let res = ctx.write_single_register(*addr, *val).await;
                match res {
                    Ok(Ok(_)) => {}
                    Ok(Err(ex)) => {
                        let msg = format!("Write exception: {:?}", ex);
                        self.emit_error(&msg).await;
                        return Err(msg);
                    }
                    Err(e) => {
                        let msg = format!("Write failed: {}", e);
                        self.emit_error(&msg).await;
                        return Err(msg);
                    }
                }
            }
            Ok(())
        }
    }

    async fn execute_poll(&mut self) {
        let (ctx, config) = match (self.client.as_mut(), self.config.as_ref()) {
            (Some(c), Some(cfg)) => (c, cfg),
            _ => return,
        };

        let fc_normalized = config.function_code.trim();
        let is_coil = is_coil_function(fc_normalized);
        let max_count = if is_coil { 2000 } else { 125 };

        if config.count == 0
            || config.count > max_count
            || (config.start_address as u32 + config.count as u32) > 65536
        {
            self.is_polling = false;
            let err_msg = format!(
                "Invalid poll range: start={}, count={} exceeds Modbus bounds (max: {})",
                config.start_address, config.count, max_count
            );
            let _ = self.event_sender.send(SessionEvent::PollError(err_msg)).await;
            return;
        }

        ctx.set_slave(Slave(config.unit_id));

        let timeout_dur = Duration::from_millis(config.timeout_ms.unwrap_or(1000).clamp(300, 10000));
        let start_addr = config.start_address;
        let count = config.count;

        let poll_fut = async {
            let exception = |ex: ExceptionCode| format!("Modbus exception: {:?}", ex);
            let words: Vec<u16> = match fc_normalized {
                "0x01" => ctx
                    .read_coils(start_addr, count)
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(exception)?
                    .into_iter()
                    .map(u16::from)
                    .collect(),
                "0x02" => ctx
                    .read_discrete_inputs(start_addr, count)
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(exception)?
                    .into_iter()
                    .map(u16::from)
                    .collect(),
                "0x03" => ctx
                    .read_holding_registers(start_addr, count)
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(exception)?,
                "0x04" => ctx
                    .read_input_registers(start_addr, count)
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(exception)?,
                _ => return Err(format!("Unsupported function code {}", fc_normalized)),
            };
            Ok::<HashMap<u16, u16>, String>(
                words
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (start_addr + i as u16, v))
                    .collect(),
            )
        };

        match time::timeout(timeout_dur, poll_fut).await {
            Ok(Ok(data_map)) => {
                let _ = self.event_sender.send(SessionEvent::DataUpdated(data_map)).await;
            }
            Ok(Err(err_msg)) => {
                let is_fatal = err_msg.contains("Broken pipe")
                    || err_msg.contains("Connection reset")
                    || err_msg.contains("Connection aborted")
                    || err_msg.contains("not connected");

                if is_fatal {
                    self.is_polling = false;
                    self.set_state(FsmState::Error(format!("Connection lost: {}", err_msg))).await;
                    if let Some(mut ctx) = self.client.take() {
                        let _ = ctx.disconnect().await;
                    }
                } else {
                    let _ = self.event_sender.send(SessionEvent::PollError(format!("Read failed: {}", err_msg))).await;
                }
            }
            Err(_) => {
                let _ = self.event_sender.send(SessionEvent::PollError(format!("Read timeout ({}ms)", timeout_dur.as_millis()))).await;
            }
        }
    }

    async fn execute_auto_increment(&mut self) {
        let config = match &self.auto_inc_config {
            Some(cfg) => cfg.clone(),
            None => return,
        };

        if config.count == 0 {
            return;
        }

        let is_coil = is_coil_function(&config.function_code);
        let is_32 = !is_coil && is_32bit_datatype(&config.data_type);
        let step = if is_32 { 2 } else { 1 };

        let mut update_map = HashMap::new();
        {
            let mut mem = self.memory.write().unwrap();
            let start = config.start_address;
            let end = (start as u32 + config.count as u32).min(65536) as u16;

            let mut addr = start;
            while addr < end {
                if is_coil {
                    let cur = *mem.get(&addr).unwrap_or(&0);
                    let next = if cur == 0 { 1 } else { 0 };
                    mem.insert(addr, next);
                    update_map.insert(addr, next);
                } else if is_32 {
                    if addr + 1 >= end {
                        break;
                    }
                    let raw1 = *mem.get(&addr).unwrap_or(&0);
                    let raw2 = *mem.get(&(addr + 1)).unwrap_or(&0);
                    let raw_bytes = [
                        (raw1 >> 8) as u8,
                        (raw1 & 0xff) as u8,
                        (raw2 >> 8) as u8,
                        (raw2 & 0xff) as u8,
                    ];
                    let swapped = swap_bytes_32(raw_bytes, &config.byte_order);

                    let (w1, w2) = match config.data_type.as_str() {
                        "Float32" => {
                            let val = f32::from_be_bytes(swapped);
                            let next = if val.is_nan() || val.is_infinite() { 1.0 } else { val + 1.0 };
                            let out_bytes = swap_bytes_32(next.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                        "Int32" => {
                            let val = i32::from_be_bytes(swapped);
                            let next = val.wrapping_add(1);
                            let out_bytes = swap_bytes_32(next.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                        _ => {
                            let val = u32::from_be_bytes(swapped);
                            let next = val.wrapping_add(1);
                            let out_bytes = swap_bytes_32(next.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                    };

                    mem.insert(addr, w1);
                    mem.insert(addr + 1, w2);
                    update_map.insert(addr, w1);
                    update_map.insert(addr + 1, w2);
                } else if config.data_type == "Int16" {
                    let cur = *mem.get(&addr).unwrap_or(&0) as i16;
                    let next = cur.wrapping_add(1) as u16;
                    mem.insert(addr, next);
                    update_map.insert(addr, next);
                } else {
                    let cur = *mem.get(&addr).unwrap_or(&0);
                    let next = cur.wrapping_add(1);
                    mem.insert(addr, next);
                    update_map.insert(addr, next);
                }

                addr = addr.saturating_add(step);
            }
        }

        self.emit_data(update_map).await;
    }

    async fn execute_fill_random(&mut self, config: &DataGenConfig) -> Result<(), String> {
        if config.count == 0 {
            return Ok(());
        }

        let is_coil = is_coil_function(&config.function_code);
        let is_32 = !is_coil && is_32bit_datatype(&config.data_type);
        let step = if is_32 { 2 } else { 1 };

        let mut rng = SimpleRng::new();
        let mut update_map = HashMap::new();
        {
            let mut mem = self.memory.write().unwrap();
            let start = config.start_address;
            let end = (start as u32 + config.count as u32).min(65536) as u16;

            let mut addr = start;
            while addr < end {
                if is_coil {
                    let val = (rng.next_u64() % 2) as u16;
                    mem.insert(addr, val);
                    update_map.insert(addr, val);
                } else if is_32 {
                    if addr + 1 >= end {
                        break;
                    }
                    let (w1, w2) = match config.data_type.as_str() {
                        "Float32" => {
                            let val = rng.gen_f32(-5000.0, 5000.0);
                            let rounded = (val * 100.0).round() / 100.0;
                            let out_bytes = swap_bytes_32(rounded.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                        "Int32" => {
                            let val = rng.gen_i32(-100000, 100000);
                            let out_bytes = swap_bytes_32(val.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                        _ => {
                            let val = rng.gen_u32(200000);
                            let out_bytes = swap_bytes_32(val.to_be_bytes(), &config.byte_order);
                            bytes_to_words(out_bytes)
                        }
                    };
                    mem.insert(addr, w1);
                    mem.insert(addr + 1, w2);
                    update_map.insert(addr, w1);
                    update_map.insert(addr + 1, w2);
                } else {
                    let val = rng.gen_range_u16(1000);
                    mem.insert(addr, val);
                    update_map.insert(addr, val);
                }

                addr = addr.saturating_add(step);
            }
        }

        self.emit_data(update_map).await;
        Ok(())
    }
}

#[inline]
fn is_coil_function(fc: &str) -> bool {
    matches!(fc.trim(), "0x01" | "0x02")
}

#[inline]
fn is_readonly_function(fc: &str) -> bool {
    matches!(fc.trim(), "0x02" | "0x04")
}

#[inline]
fn is_32bit_datatype(dt: &str) -> bool {
    matches!(dt, "Float32" | "Int32" | "UInt32")
}

#[inline]
fn bytes_to_words(b: [u8; 4]) -> (u16, u16) {
    (u16::from_be_bytes([b[0], b[1]]), u16::from_be_bytes([b[2], b[3]]))
}

#[inline]
fn swap_bytes_32(bytes: [u8; 4], byte_order: &str) -> [u8; 4] {
    let [a, b, c, d] = bytes;
    match byte_order {
        "CDAB" => [c, d, a, b],
        "BADC" => [b, a, d, c],
        "DCBA" => [d, c, b, a],
        _ => [a, b, c, d],
    }
}

struct SimpleRng(u64);

impl SimpleRng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_range_u16(&mut self, max: u16) -> u16 {
        if max == 0 {
            return 0;
        }
        (self.next_u64() % (max as u64)) as u16
    }

    fn gen_f32(&mut self, min: f32, max: f32) -> f32 {
        let norm = (self.next_u64() & 0xffffff) as f32 / 16777215.0;
        min + norm * (max - min)
    }

    fn gen_i32(&mut self, min: i32, max: i32) -> i32 {
        let range = (max as i64 - min as i64 + 1) as u64;
        if range == 0 {
            return min;
        }
        let val = (self.next_u64() % range) as i64;
        (min as i64 + val) as i32
    }

    fn gen_u32(&mut self, max: u32) -> u32 {
        (self.next_u64() % (max as u64 + 1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn send_cmd(tx: &mpsc::Sender<CommandEnvelope>, cmd: SessionCommand) {
        tx.send(cmd.into()).await.unwrap();
    }

    #[test]
    fn test_client_drop_guard() {
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let client_id = 1;
        clients.write().unwrap().insert(
            client_id,
            TcpClientInfo {
                id: client_id,
                address: "127.0.0.1:5020".into(),
                connected_at: 1000,
                last_request_at: None,
            },
        );

        assert_eq!(clients.read().unwrap().len(), 1);

        {
            let _guard = ClientDropGuard {
                client_id,
                clients: clients.clone(),
            };
            assert_eq!(clients.read().unwrap().len(), 1);
        }

        assert_eq!(clients.read().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_slave_high_address_boundary() {
        let memory = Arc::new(RwLock::new(HashMap::new()));
        memory.write().unwrap().insert(65535, 9999);
        let (evt_tx, _evt_rx) = mpsc::channel(32);
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let service = SlaveService {
            client_id: 1,
            _unit_id: 1,
            memory: memory.clone(),
            event_sender: evt_tx,
            clients: clients.clone(),
            _guard: Arc::new(ClientDropGuard { client_id: 1, clients }),
        };

        let res = service.call(Request::ReadHoldingRegisters(65500, 36)).await;
        assert!(res.is_ok());
        if let Ok(Response::ReadHoldingRegisters(vals)) = res {
            assert_eq!(vals.len(), 36);
            assert_eq!(vals[35], 9999);
        } else {
            panic!("Unexpected response");
        }
    }

    #[test]
    fn test_check_bounds() {
        assert_eq!(check_bounds(0, 0, 125), Err(ExceptionCode::IllegalDataValue));
        assert_eq!(check_bounds(0, 126, 125), Err(ExceptionCode::IllegalDataValue));
        assert_eq!(check_bounds(65535, 2, 125), Err(ExceptionCode::IllegalDataAddress));
        assert_eq!(check_bounds(0, 10, 125), Ok(()));
        assert_eq!(check_bounds(65530, 6, 125), Ok(()));
    }

    #[tokio::test]
    async fn test_tcp_slave_master_loopback() {
        let slave_clients = Arc::new(RwLock::new(HashMap::new()));
        let (slave_cmd_tx, slave_cmd_rx) = mpsc::channel(32);
        let (slave_evt_tx, mut slave_evt_rx) = mpsc::channel(32);
        let slave_actor = SessionActor::new("slave_test".into(), slave_cmd_rx, slave_evt_tx, slave_clients.clone());
        tokio::spawn(slave_actor.run());

        // 1. Connect Slave to 127.0.0.1:15020
        let slave_cfg = ConnectionConfig {
            protocol: "TCP".into(),
            ip: "127.0.0.1".into(),
            port: 15020,
            serial_port: "".into(),
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: "None".into(),
            role: "Slave".into(),
            unit_id: 1,
            function_code: "0x03".into(),
            start_address: 0,
            count: 20,
            timeout_ms: Some(1000),
        };
        send_cmd(&slave_cmd_tx, SessionCommand::Connect(slave_cfg)).await;

        let evt = slave_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::StateChanged(FsmState::Connecting));
        let evt = slave_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::StateChanged(FsmState::Connected));

        // 2. Connect Master to 127.0.0.1:15020
        let master_clients = Arc::new(RwLock::new(HashMap::new()));
        let (master_cmd_tx, master_cmd_rx) = mpsc::channel(32);
        let (master_evt_tx, mut master_evt_rx) = mpsc::channel(32);
        let master_actor = SessionActor::new("master_test".into(), master_cmd_rx, master_evt_tx, master_clients);
        tokio::spawn(master_actor.run());

        let master_cfg = ConnectionConfig {
            protocol: "TCP".into(),
            ip: "127.0.0.1".into(),
            port: 15020,
            serial_port: "".into(),
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: "None".into(),
            role: "Master".into(),
            unit_id: 1,
            function_code: "0x03".into(),
            start_address: 0,
            count: 10,
            timeout_ms: Some(1000),
        };
        send_cmd(&master_cmd_tx, SessionCommand::Connect(master_cfg.clone())).await;

        let evt = master_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::StateChanged(FsmState::Connecting));
        let evt = master_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::StateChanged(FsmState::Connected));

        // Wait brief tick for connection handshake
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify Slave sees 1 connected client
        assert_eq!(slave_clients.read().unwrap().len(), 1);

        // 3. Master writes register 5 = 1234
        send_cmd(&master_cmd_tx, SessionCommand::WriteRegister(WritePayload { address: 5, value: 1234 })).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // 4. Master reads registers
        send_cmd(&master_cmd_tx, SessionCommand::ReadOnce).await;
        let evt = master_evt_rx.recv().await.unwrap();
        match evt {
            SessionEvent::DataUpdated(map) => {
                assert_eq!(map.get(&5), Some(&1234));
            }
            other => panic!("Expected DataUpdated, got {:?}", other),
        }

        // 5. Test constraint: Write and ReadOnce rejected while polling is active
        send_cmd(&master_cmd_tx, SessionCommand::StartPolling(100)).await;
        send_cmd(&master_cmd_tx, SessionCommand::WriteRegister(WritePayload { address: 6, value: 5678 })).await;
        let evt = master_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Cannot write while polling is active".into()));

        send_cmd(&master_cmd_tx, SessionCommand::ReadOnce).await;
        let evt = master_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Cannot read once while polling is active".into()));

        send_cmd(&master_cmd_tx, SessionCommand::StopPolling).await;

        // 6. Test constraint: Write rejected for read-only FC 0x04
        send_cmd(&master_cmd_tx, SessionCommand::UpdateConfig(ConnectionConfig {
            function_code: "0x04".into(),
            ..master_cfg
        })).await;
        send_cmd(&master_cmd_tx, SessionCommand::WriteRegister(WritePayload { address: 6, value: 5678 })).await;
        let evt = master_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Function code 0x04 is read-only".into()));

        // 7. Clean up
        send_cmd(&slave_cmd_tx, SessionCommand::Disconnect).await;
        send_cmd(&master_cmd_tx, SessionCommand::Disconnect).await;
    }

    #[tokio::test]
    async fn test_slave_auto_increment_and_random() {
        let slave_clients = Arc::new(RwLock::new(HashMap::new()));
        let (slave_cmd_tx, slave_cmd_rx) = mpsc::channel(32);
        let (slave_evt_tx, mut slave_evt_rx) = mpsc::channel(32);
        let slave_actor = SessionActor::new("slave_inc_test".into(), slave_cmd_rx, slave_evt_tx, slave_clients);
        tokio::spawn(slave_actor.run());

        // Connect Slave
        let slave_cfg = ConnectionConfig {
            protocol: "TCP".into(),
            ip: "127.0.0.1".into(),
            port: 15022,
            serial_port: "".into(),
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: "None".into(),
            role: "Slave".into(),
            unit_id: 1,
            function_code: "0x03".into(),
            start_address: 0,
            count: 20,
            timeout_ms: Some(1000),
        };
        send_cmd(&slave_cmd_tx, SessionCommand::Connect(slave_cfg)).await;
        let _ = slave_evt_rx.recv().await.unwrap(); // Connecting
        let _ = slave_evt_rx.recv().await.unwrap(); // Connected

        // 1. Test FillRandom
        let gen_cfg = DataGenConfig {
            start_address: 0,
            count: 10,
            function_code: "0x03".into(),
            data_type: "Int16".into(),
            byte_order: "ABCD".into(),
        };
        send_cmd(&slave_cmd_tx, SessionCommand::FillRandom(gen_cfg.clone())).await;
        let evt = slave_evt_rx.recv().await.unwrap();
        match evt {
            SessionEvent::DataUpdated(map) => {
                assert_eq!(map.len(), 10);
                for i in 0..10 {
                    assert!(map.contains_key(&i));
                }
            }
            other => panic!("Expected DataUpdated from FillRandom, got {:?}", other),
        }

        // 2. Test StartAutoIncrement
        send_cmd(&slave_cmd_tx, SessionCommand::StartAutoIncrement {
            interval_ms: 100,
            config: gen_cfg,
        }).await;

        // 3. Test constraint: FillRandom rejected while auto-increment is active
        send_cmd(&slave_cmd_tx, SessionCommand::FillRandom(DataGenConfig {
            start_address: 0,
            count: 5,
            function_code: "0x03".into(),
            data_type: "Int16".into(),
            byte_order: "ABCD".into(),
        })).await;
        let evt = slave_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Cannot fill random while auto-increment is active".into()));

        // 4. Test constraint: Write rejected while auto-increment is active
        send_cmd(&slave_cmd_tx, SessionCommand::WriteRegister(WritePayload { address: 1, value: 99 })).await;
        let evt = slave_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Cannot write while auto-increment is active".into()));

        // Receive tick update from auto increment
        let evt = tokio::time::timeout(Duration::from_millis(500), slave_evt_rx.recv()).await.unwrap().unwrap();
        match evt {
            SessionEvent::DataUpdated(map) => {
                assert_eq!(map.len(), 10);
            }
            other => panic!("Expected DataUpdated from AutoIncrement, got {:?}", other),
        }

        // 5. Stop auto increment
        send_cmd(&slave_cmd_tx, SessionCommand::StopAutoIncrement).await;

        // 6. Test constraint: Auto-increment rejected for read-only FC 0x04
        send_cmd(&slave_cmd_tx, SessionCommand::StartAutoIncrement {
            interval_ms: 100,
            config: DataGenConfig {
                start_address: 0,
                count: 5,
                function_code: "0x04".into(),
                data_type: "Int16".into(),
                byte_order: "ABCD".into(),
            },
        }).await;
        let evt = slave_evt_rx.recv().await.unwrap();
        assert_eq!(evt, SessionEvent::PollError("Function code 0x04 is read-only".into()));

        send_cmd(&slave_cmd_tx, SessionCommand::Disconnect).await;
    }
}
