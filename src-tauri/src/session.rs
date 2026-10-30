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
}

#[derive(Debug, Clone, Serialize)]
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
                    let values: Vec<u16> = (start..start + count)
                        .map(|addr| *mem.get(&addr).unwrap_or(&0))
                        .collect();
                    Response::ReadHoldingRegisters(values)
                })
            }
            Request::ReadInputRegisters(start, count) => {
                check_bounds(start, count, 125).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<u16> = (start..start + count)
                        .map(|addr| *mem.get(&addr).unwrap_or(&0))
                        .collect();
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
                    let values: Vec<bool> = (start..start + count)
                        .map(|addr| mem.get(&addr).copied().unwrap_or(0) != 0)
                        .collect();
                    Response::ReadCoils(values)
                })
            }
            Request::ReadDiscreteInputs(start, count) => {
                check_bounds(start, count, 2000).map(|_| {
                    let mem = self.memory.read().unwrap();
                    let values: Vec<bool> = (start..start + count)
                        .map(|addr| mem.get(&addr).copied().unwrap_or(0) != 0)
                        .collect();
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
    receiver: mpsc::Receiver<SessionCommand>,
    event_sender: mpsc::Sender<SessionEvent>,
    config: Option<ConnectionConfig>,
    client: Option<ModbusContext>,
    slave_stop: Option<tokio::sync::oneshot::Sender<()>>,
    memory: Arc<RwLock<HashMap<u16, u16>>>,
    clients: Arc<RwLock<HashMap<usize, TcpClientInfo>>>,
    is_polling: bool,
    poll_interval: u64,
}

impl SessionActor {
    pub fn new(
        id: String,
        receiver: mpsc::Receiver<SessionCommand>,
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
        let mut interval = time::interval(Duration::from_millis(self.poll_interval));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                cmd_opt = self.receiver.recv() => {
                    match cmd_opt {
                        Some(cmd) => self.handle_command(cmd, &mut interval).await,
                        None => break, // Channel dropped by frontend, exit cleanly
                    }
                }
                _ = interval.tick(), if self.is_polling && self.state == FsmState::Connected => {
                    self.execute_poll().await;
                }
            }
        }
        self.cleanup().await;
        let _ = self.set_state(FsmState::Disconnected).await;
    }

    async fn cleanup(&mut self) {
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

    async fn handle_command(&mut self, cmd: SessionCommand, interval: &mut time::Interval) {
        match cmd {
            SessionCommand::Connect(config) => {
                self.is_polling = false;
                self.cleanup().await;

                self.config = Some(config.clone());
                self.set_state(FsmState::Connecting).await;

                // --- SLAVE 模式 ---
                if config.role == "Slave" {
                    if config.protocol == "TCP" {
                        let bind_addr = format!("{}:{}", config.ip, config.port);
                        match tokio::net::TcpListener::bind(&bind_addr).await {
                            Ok(listener) => {
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
                                    let _ = server.serve_until(
                                        &on_connected,
                                        |_err| {},
                                        async move {
                                            let _ = stop_rx.await;
                                        },
                                    ).await;
                                });
                            }
                            Err(e) => {
                                self.set_state(FsmState::Error(format!("Bind {} failed: {}", bind_addr, e))).await;
                            }
                        }
                    } else {
                        // RTU Slave
                        self.set_state(FsmState::Error("RTU Slave not supported on this platform".into())).await;
                    }
                    return;
                }

                // --- MASTER 模式 ---
                if config.protocol == "TCP" {
                    let addr_str = format!("{}:{}", config.ip, config.port);
                    if let Ok(addr) = addr_str.parse() {
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
                    } else {
                        self.set_state(FsmState::Error(format!("Invalid IP:Port '{}'", addr_str))).await;
                    }
                } else {
                    // RTU Master
                    let builder = tokio_serial::new(&config.serial_port, config.baud_rate)
                        .data_bits(match config.data_bits { 7 => tokio_serial::DataBits::Seven, _ => tokio_serial::DataBits::Eight })
                        .stop_bits(match config.stop_bits { 2 => tokio_serial::StopBits::Two, _ => tokio_serial::StopBits::One })
                        .parity(match config.parity.as_str() {
                            "Even" => tokio_serial::Parity::Even,
                            "Odd" => tokio_serial::Parity::Odd,
                            _ => tokio_serial::Parity::None,
                        });
                    match builder.open_native_async() {
                        Ok(port) => {
                            let mut ctx = tokio_modbus::client::rtu::attach_slave(port, Slave(config.unit_id));
                            ctx.set_slave(Slave(config.unit_id));
                            self.client = Some(ctx);
                            self.set_state(FsmState::Connected).await;
                        }
                        Err(e) => {
                            self.set_state(FsmState::Error(format!("Open serial {} failed: {}", config.serial_port, e))).await;
                        }
                    }
                }
            }
            SessionCommand::Disconnect => {
                self.is_polling = false;
                self.cleanup().await;
                self.set_state(FsmState::Disconnected).await;
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
            }
            SessionCommand::StartPolling(interval_ms) => {
                if self.state == FsmState::Connected {
                    self.poll_interval = interval_ms;
                    self.is_polling = true;
                    *interval = time::interval(Duration::from_millis(interval_ms));
                    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                }
            }
            SessionCommand::StopPolling => {
                self.is_polling = false;
            }
            SessionCommand::ReadOnce => {
                if self.state == FsmState::Connected {
                    self.execute_poll().await;
                }
            }
            SessionCommand::WriteRegister(payload) => {
                if let Some(cfg) = &self.config {
                    if cfg.role == "Slave" {
                        {
                            let mut mem = self.memory.write().unwrap();
                            mem.insert(payload.address, payload.value);
                        }
                        let mut map = HashMap::new();
                        map.insert(payload.address, payload.value);
                        let _ = self.event_sender.send(SessionEvent::DataUpdated(map)).await;
                        return;
                    }
                }

                if let Some(ctx) = &mut self.client {
                    let is_coil = self.config.as_ref().map(|c| c.function_code == "0x01" || c.function_code == "0x02").unwrap_or(false);
                    let write_res = if is_coil {
                        ctx.write_single_coil(payload.address, payload.value != 0).await
                    } else {
                        ctx.write_single_register(payload.address, payload.value).await
                    };

                    match write_res {
                        Ok(Ok(_)) => {}
                        Ok(Err(ex)) => {
                            let _ = self
                                .event_sender
                                .send(SessionEvent::PollError(format!("Write exception: {:?}", ex)))
                                .await;
                        }
                        Err(e) => {
                            let _ = self
                                .event_sender
                                .send(SessionEvent::PollError(format!("Write failed: {}", e)))
                                .await;
                        }
                    }
                }
            }
            SessionCommand::WriteRegistersBatch(batch) => {
                if let Some(cfg) = &self.config {
                    if cfg.role == "Slave" {
                        {
                            let mut mem = self.memory.write().unwrap();
                            for (addr, val) in &batch {
                                mem.insert(*addr, *val);
                            }
                        }
                        let _ = self.event_sender.send(SessionEvent::DataUpdated(batch)).await;
                    } else if let Some(ctx) = &mut self.client {
                        let is_coil = cfg.function_code == "0x01" || cfg.function_code == "0x02";
                        if is_coil {
                            for (addr, val) in &batch {
                                let _ = ctx.write_single_coil(*addr, *val != 0).await;
                            }
                        } else {
                            let mut addrs: Vec<u16> = batch.keys().copied().collect();
                            addrs.sort_unstable();
                            if !addrs.is_empty() {
                                let start = addrs[0];
                                let count = addrs.len() as u16;
                                let is_contiguous = addrs.last().copied().unwrap_or(0) == start + count - 1;
                                if is_contiguous && count <= 123 {
                                    let vals: Vec<u16> = addrs.iter().map(|a| batch[a]).collect();
                                    let _ = ctx.write_multiple_registers(start, &vals).await;
                                } else {
                                    for (addr, val) in &batch {
                                        let _ = ctx.write_single_register(*addr, *val).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn execute_poll(&mut self) {
        let (ctx, config) = match (self.client.as_mut(), self.config.as_ref()) {
            (Some(c), Some(cfg)) => (c, cfg),
            _ => return,
        };

        let fc_lower = config.function_code.trim().to_lowercase();
        let fc_normalized = match fc_lower.as_str() {
            "0x01" | "1" | "01" | "coils" => "0x01",
            "0x02" | "2" | "02" | "discrete_inputs" => "0x02",
            "0x03" | "3" | "03" | "holding" | "holding_registers" => "0x03",
            "0x04" | "4" | "04" | "input_registers" => "0x04",
            other => other,
        };

        let is_coil = fc_normalized == "0x01" || fc_normalized == "0x02";
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
            match fc_normalized {
                "0x01" => {
                    let res = ctx.read_coils(start_addr, count).await.map_err(|e| e.to_string())?;
                    match res {
                        Ok(bits) => {
                            let mut data_map = HashMap::new();
                            for (i, bit) in bits.into_iter().enumerate() {
                                data_map.insert(start_addr + i as u16, if bit { 1 } else { 0 });
                            }
                            Ok(data_map)
                        }
                        Err(ex) => Err(format!("Modbus exception: {:?}", ex)),
                    }
                }
                "0x02" => {
                    let res = ctx.read_discrete_inputs(start_addr, count).await.map_err(|e| e.to_string())?;
                    match res {
                        Ok(bits) => {
                            let mut data_map = HashMap::new();
                            for (i, bit) in bits.into_iter().enumerate() {
                                data_map.insert(start_addr + i as u16, if bit { 1 } else { 0 });
                            }
                            Ok(data_map)
                        }
                        Err(ex) => Err(format!("Modbus exception: {:?}", ex)),
                    }
                }
                "0x03" => {
                    let res = ctx.read_holding_registers(start_addr, count).await.map_err(|e| e.to_string())?;
                    match res {
                        Ok(words) => {
                            let mut data_map = HashMap::new();
                            for (i, val) in words.into_iter().enumerate() {
                                data_map.insert(start_addr + i as u16, val);
                            }
                            Ok(data_map)
                        }
                        Err(ex) => Err(format!("Modbus exception: {:?}", ex)),
                    }
                }
                "0x04" => {
                    let res = ctx.read_input_registers(start_addr, count).await.map_err(|e| e.to_string())?;
                    match res {
                        Ok(words) => {
                            let mut data_map = HashMap::new();
                            for (i, val) in words.into_iter().enumerate() {
                                data_map.insert(start_addr + i as u16, val);
                            }
                            Ok(data_map)
                        }
                        Err(ex) => Err(format!("Modbus exception: {:?}", ex)),
                    }
                }
                _ => Err(format!("Unsupported function code {}", fc_normalized)),
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_check_bounds() {
        assert_eq!(check_bounds(0, 0, 125), Err(ExceptionCode::IllegalDataValue));
        assert_eq!(check_bounds(0, 126, 125), Err(ExceptionCode::IllegalDataValue));
        assert_eq!(check_bounds(65535, 2, 125), Err(ExceptionCode::IllegalDataAddress));
        assert_eq!(check_bounds(0, 10, 125), Ok(()));
        assert_eq!(check_bounds(65530, 6, 125), Ok(()));
    }
}
