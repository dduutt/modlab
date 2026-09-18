mod modbus;
#[cfg(test)]
use modbus::client::connect_tcp;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Mutex as AsyncMutex;
use tokio_modbus::client::Context as ModbusContext;
use tokio_modbus::prelude::*;

use modbus::client::{connect_rtu_logged, connect_tcp_logged, open_serial};
use modbus::rtu_router::{SerialPortRouter, SlaveRoute};
use modbus::server::SessionMemory;
use modbus::tcp_router::TcpSlaveRegistry;
use modbus::types::ConnectionConfig;
use modbus::wire::{emit_frame, WireSink};

struct AppState {
    sessions: Mutex<HashMap<String, SessionMemory>>,
    connection_status: Mutex<HashMap<String, String>>,
    clients: Mutex<HashMap<String, Arc<ClientConnection>>>,
    tcp_slaves: AsyncMutex<TcpSlaveRegistry>,
    connection_operations: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
    rtu_routers: Mutex<HashMap<String, Arc<SerialPortRouter>>>,
    session_rtu_bindings: Mutex<HashMap<String, (String, u8)>>,
}

struct ClientConnection {
    context: AsyncMutex<Option<ModbusContext>>,
    config: ConnectionConfig,
    closing: AtomicBool,
    wire_sink: Option<WireSink>,
}

enum ClientRequest<'a> {
    Read {
        function_code: &'a str,
        start: u16,
        count: u16,
    },
    Write {
        function_code: &'a str,
        address: u16,
        value: u16,
    },
    WriteMultiple {
        function_code: &'a str,
        start: u16,
        values: &'a [u16],
    },
}

impl ClientConnection {
    fn new(context: ModbusContext, config: ConnectionConfig) -> Self {
        Self {
            context: AsyncMutex::new(Some(context)),
            config,
            closing: AtomicBool::new(false),
            wire_sink: None,
        }
    }

    async fn execute(
        &self,
        request: ClientRequest<'_>,
        unit_id: u8,
        timeout_ms: u32,
    ) -> Result<Vec<u16>, String> {
        let mut slot = self.context.lock().await;
        if self.closing.load(Ordering::SeqCst) {
            return Err("Modbus connection is closing".into());
        }
        if slot.is_none() {
            // The outer request loop owns the retry budget. Reconnect once per attempt.
            *slot = Some(
                connect_tcp_logged(
                    &self.config.ip,
                    self.config.port,
                    unit_id,
                    timeout_ms,
                    0,
                    self.wire_sink.clone(),
                )
                .await?,
            );
        }
        let context = slot.as_mut().ok_or("Modbus connection is unavailable")?;
        let operation = async {
            match request {
                ClientRequest::Read {
                    function_code,
                    start,
                    count,
                } => {
                    let values = read_remote(context, function_code, start, count, unit_id).await?;
                    validate_read_response(&values, count)?;
                    Ok(values)
                }
                ClientRequest::Write {
                    function_code,
                    address,
                    value,
                } => {
                    write_remote(context, function_code, address, value, unit_id).await?;
                    Ok(Vec::new())
                }
                ClientRequest::WriteMultiple {
                    function_code,
                    start,
                    values,
                } => {
                    write_remote_multiple(context, function_code, start, values, unit_id).await?;
                    Ok(Vec::new())
                }
            }
        };
        let result =
            match tokio::time::timeout(Duration::from_millis(u64::from(timeout_ms)), operation)
                .await
            {
                Ok(result) => result,
                Err(_) => Err(format!("request timed out after {timeout_ms}ms")),
            };
        if result.is_err() && self.config.protocol == "TCP" {
            // Drop even on the final failed attempt, so the next poll starts clean.
            // Late replies belong to the old socket and cannot poison the next request.
            *slot = None;
        }
        result
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            connection_status: Mutex::new(HashMap::new()),
            clients: Mutex::new(HashMap::new()),
            tcp_slaves: AsyncMutex::new(TcpSlaveRegistry::default()),
            connection_operations: Mutex::new(HashMap::new()),
            rtu_routers: Mutex::new(HashMap::new()),
            session_rtu_bindings: Mutex::new(HashMap::new()),
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex
        .lock()
        .map_err(|_| "Application state lock poisoned".into())
}

fn validate_request(function_code: &str, start: u16, count: u16) -> Result<(), String> {
    let max_count = match function_code {
        "0x01" | "0x02" => 2_000,
        "0x03" | "0x04" => 125,
        _ => return Err(format!("Unsupported function code: {function_code}")),
    };

    if count == 0 {
        return Err("Count must be at least 1".into());
    }
    if count > max_count {
        return Err(format!(
            "Count {count} exceeds the Modbus limit of {max_count} for {function_code}"
        ));
    }
    if u32::from(start) + u32::from(count) > 65_536 {
        return Err(format!(
            "Address range {start}..{} exceeds 65535",
            u32::from(start) + u32::from(count) - 1
        ));
    }
    Ok(())
}

fn validate_communication_settings(timeout_ms: u32, retries: u8) -> Result<(), String> {
    if !(100..=10_000).contains(&timeout_ms) {
        return Err("Timeout must be between 100ms and 10000ms".into());
    }
    if retries > 10 {
        return Err("Retries must be between 0 and 10".into());
    }
    Ok(())
}

fn validate_multiple_write(function_code: &str, start: u16, count: usize) -> Result<(), String> {
    let max_count = match function_code {
        "0x01" | "0x02" => 1_968,
        "0x03" | "0x04" => 123,
        _ => return Err(format!("Unsupported function code: {function_code}")),
    };
    if count == 0 || count > max_count {
        return Err(format!("Write count must be between 1 and {max_count}"));
    }
    validate_request(function_code, start, count as u16)
}

async fn write_remote_multiple(
    context: &mut ModbusContext,
    function_code: &str,
    start: u16,
    values: &[u16],
    unit_id: u8,
) -> Result<(), String> {
    validate_multiple_write(function_code, start, values.len())?;
    context.set_slave(Slave(unit_id));
    match function_code {
        "0x01" => {
            let coils: Vec<bool> = values.iter().map(|&value| value != 0).collect();
            context
                .write_multiple_coils(start, &coils)
                .await
                .map_err(|err| err.to_string())
        }
        "0x03" => context
            .write_multiple_registers(start, values)
            .await
            .map_err(|err| err.to_string()),
        "0x02" | "0x04" => Err(format!(
            "{function_code} is read-only and cannot be written"
        )),
        _ => Err(format!("Unsupported function code: {function_code}")),
    }
}

fn session_memory(state: &AppState, session_id: &str) -> Result<SessionMemory, String> {
    let mut sessions = lock(&state.sessions)?;
    Ok(sessions
        .entry(session_id.to_string())
        .or_insert_with(SessionMemory::new)
        .clone())
}

fn connected_memory(state: &AppState, session_id: &str) -> Result<SessionMemory, String> {
    if !lock(&state.connection_status)?.contains_key(session_id) {
        return Err("Session is disconnected".into());
    }
    session_memory(state, session_id)
}

fn validate_read_response(values: &[u16], count: u16) -> Result<(), String> {
    if values.len() != usize::from(count) {
        return Err(format!(
            "Invalid read response: expected {count} values, received {}",
            values.len()
        ));
    }
    Ok(())
}

fn cached_client(
    state: &AppState,
    session_id: &str,
) -> Result<Option<Arc<ClientConnection>>, String> {
    Ok(lock(&state.clients)?.get(session_id).cloned())
}

fn connection_operation(state: &AppState, session_id: &str) -> Result<Arc<AsyncMutex<()>>, String> {
    Ok(lock(&state.connection_operations)?
        .entry(session_id.to_owned())
        .or_insert_with(|| Arc::new(AsyncMutex::new(())))
        .clone())
}

async fn close_client(client: Arc<ClientConnection>) {
    client.closing.store(true, Ordering::SeqCst);
    let mut context = client.context.lock().await;
    // The library uses a disconnect request to release the client service. The
    // request itself is expected to report an error after the transport closes.
    if let Some(mut context) = context.take() {
        let _ = context.disconnect().await;
    }
}

async fn read_remote(
    context: &mut ModbusContext,
    function_code: &str,
    start: u16,
    count: u16,
    unit_id: u8,
) -> Result<Vec<u16>, String> {
    context.set_slave(Slave(unit_id));
    match function_code {
        "0x01" => context
            .read_coils(start, count)
            .await
            .map(|values| values.into_iter().map(u16::from).collect())
            .map_err(|err| err.to_string()),
        "0x02" => context
            .read_discrete_inputs(start, count)
            .await
            .map(|values| values.into_iter().map(u16::from).collect())
            .map_err(|err| err.to_string()),
        "0x03" => context
            .read_holding_registers(start, count)
            .await
            .map_err(|err| err.to_string()),
        "0x04" => context
            .read_input_registers(start, count)
            .await
            .map_err(|err| err.to_string()),
        _ => Err(format!("Unsupported function code: {function_code}")),
    }
}

async fn write_remote(
    context: &mut ModbusContext,
    function_code: &str,
    address: u16,
    value: u16,
    unit_id: u8,
) -> Result<(), String> {
    context.set_slave(Slave(unit_id));
    match function_code {
        "0x01" => context
            .write_single_coil(address, value != 0)
            .await
            .map_err(|err| err.to_string()),
        "0x03" => context
            .write_single_register(address, value)
            .await
            .map_err(|err| err.to_string()),
        "0x02" | "0x04" => Err(format!(
            "{function_code} is read-only and cannot be written"
        )),
        _ => Err(format!("Unsupported function code: {function_code}")),
    }
}

#[tauri::command]
async fn connect_modbus_native(
    app: tauri::AppHandle,
    session_id: String,
    session_title: Option<String>,
    config: ConnectionConfig,
    unit_id: u8,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let operation = connection_operation(&state, &session_id)?;
    let _operation = operation.lock().await;
    if lock(&state.connection_status)?.contains_key(&session_id) {
        return Err("Session is already connected; disconnect before starting it again".into());
    }
    validate_communication_settings(config.timeout_ms, config.retries)?;
    if !matches!(config.role.as_str(), "Master" | "Slave") {
        return Err(format!("Unsupported role: {}", config.role));
    }
    let target = if config.protocol == "TCP" {
        format!("{}:{}", config.ip, config.port)
    } else {
        format!("{} ({})", config.serial_port, config.baud_rate)
    };
    let memory = session_memory(&state, &session_id)?;

    if config.role == "Master" {
        let log_app = app.clone();
        let log_session = session_id.clone();
        let tcp = config.protocol == "TCP";
        let sink: WireSink = Arc::new(move |direction, bytes, complete| {
            emit_frame(&log_app, &log_session, direction, bytes, tcp, complete)
        });
        let context = match config.protocol.as_str() {
            "TCP" => {
                if config.ip == "0.0.0.0" {
                    return Err("Invalid target IP '0.0.0.0'. Please enter a Slave IP (for example 127.0.0.1).".into());
                }
                connect_tcp_logged(
                    &config.ip,
                    config.port,
                    unit_id,
                    config.timeout_ms,
                    config.retries,
                    Some(sink.clone()),
                )
                .await?
            }
            "RTU" => connect_rtu_logged(&config, unit_id, sink.clone())?,
            _ => return Err(format!("Unsupported protocol: {}", config.protocol)),
        };
        lock(&state.clients)?.insert(
            session_id.clone(),
            Arc::new({
                let mut client = ClientConnection::new(context, config.clone());
                client.wire_sink = Some(sink);
                client
            }),
        );
        lock(&state.connection_status)?.insert(session_id, target.clone());
        return Ok(format!(
            "Connected to Modbus {} ({target})",
            config.protocol
        ));
    }

    if config.protocol == "RTU" {
        // Unbind previous RTU binding for this session_id if any
        let old_binding = {
            let mut bindings = lock(&state.session_rtu_bindings)?;
            bindings.remove(&session_id)
        };
        if let Some((old_port, old_unit_id)) = old_binding {
            let mut routers = lock(&state.rtu_routers)?;
            if let Some(router) = routers.get(&old_port) {
                let is_empty = {
                    let mut slaves = router.slaves.lock().map_err(|_| "Router lock poisoned")?;
                    slaves.remove(&old_unit_id);
                    slaves.is_empty()
                };
                if is_empty {
                    if let Some(r) = routers.remove(&old_port) {
                        let _ = r.stop_tx.send(());
                        r.task.abort();
                    }
                }
            }
        }

        let port_name = config.serial_port.clone();
        let title = session_title.unwrap_or_else(|| session_id.clone());

        let router = {
            let mut routers = lock(&state.rtu_routers)?;
            if let Some(existing) = routers.get(&port_name) {
                if !existing.config.serial_settings_match(&config) {
                    return Err(format!("{port_name} is already open with different serial settings; baud rate, data bits, stop bits and parity must match"));
                }
                existing.clone()
            } else {
                let serial = open_serial(&config)?;
                let new_router = Arc::new(SerialPortRouter::new(
                    app.clone(),
                    port_name.clone(),
                    serial,
                    config.clone(),
                ));
                routers.insert(port_name.clone(), new_router.clone());
                new_router
            }
        };

        {
            let mut slaves = router.slaves.lock().map_err(|_| "Router lock poisoned")?;
            if let Some(existing) = slaves.get(&unit_id) {
                if existing.session_id != session_id {
                    return Err(format!(
                        "Unit ID {} is already in use on {} by session '{}'",
                        unit_id, port_name, existing.session_title
                    ));
                }
            }
            slaves.insert(
                unit_id,
                SlaveRoute {
                    session_id: session_id.clone(),
                    session_title: title,
                    memory: memory.clone(),
                },
            );
        }

        lock(&state.session_rtu_bindings)?.insert(session_id.clone(), (port_name.clone(), unit_id));
        lock(&state.connection_status)?.insert(session_id, format!("{} (ID: {})", target, unit_id));
        return Ok(format!(
            "Slave (Unit ID {unit_id}) listening on {port_name}"
        ));
    }

    if config.protocol == "TCP" {
        if config.port == 0 {
            return Err("TCP listen port must be between 1 and 65535".into());
        }
        let address: SocketAddr = target
            .parse()
            .map_err(|err| format!("Invalid listen address '{target}': {err}"))?;
        state
            .tcp_slaves
            .lock()
            .await
            .set_logger(Arc::new(move |entry| {
                use tauri::Emitter;
                let _ = app.emit("traffic-log-entry", entry);
            }))
            .register(
                address,
                unit_id,
                session_id.clone(),
                session_title.unwrap_or_else(|| session_id.clone()),
                memory,
            )
            .await?;
        lock(&state.connection_status)?.insert(session_id, format!("{target} (ID: {unit_id})"));
        return Ok(format!("Slave (Unit ID {unit_id}) listening on {target}"));
    }

    Err(format!("Unsupported protocol: {}", config.protocol))
}

#[tauri::command]
async fn disconnect_modbus_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let operation = connection_operation(&state, &session_id)?;
    let _operation = operation.lock().await;
    let client = {
        let mut clients = lock(&state.clients)?;
        clients.remove(&session_id)
    };
    if let Some(client) = client {
        close_client(client).await;
    }
    state
        .tcp_slaves
        .lock()
        .await
        .unregister(&session_id)
        .await?;

    let old_binding = {
        let mut bindings = lock(&state.session_rtu_bindings)?;
        bindings.remove(&session_id)
    };
    if let Some((port_name, unit_id)) = old_binding {
        let mut routers = lock(&state.rtu_routers)?;
        if let Some(router) = routers.get(&port_name) {
            let is_empty = {
                let mut slaves = router.slaves.lock().map_err(|_| "Router lock poisoned")?;
                slaves.remove(&unit_id);
                slaves.is_empty()
            };
            if is_empty {
                if let Some(r) = routers.remove(&port_name) {
                    let _ = r.stop_tx.send(());
                    r.task.abort();
                }
            }
        }
    }

    lock(&state.connection_status)?.remove(&session_id);
    Ok("Disconnected".into())
}

#[tauri::command]
async fn read_registers_native(
    session_id: String,
    start: u16,
    count: u16,
    function_code: String,
    unit_id: u8,
    timeout_ms: u32,
    retries: u8,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    let operation = connection_operation(&state, &session_id)?;
    let _operation = operation.lock().await;
    validate_request(&function_code, start, count)?;
    validate_communication_settings(timeout_ms, retries)?;
    let memory = connected_memory(&state, &session_id)?;

    if let Some(client) = cached_client(&state, &session_id)? {
        if client.closing.load(Ordering::SeqCst) {
            return Err("Modbus connection is closing".into());
        }
        let mut last_error = String::new();
        for _ in 0..=retries {
            let result = client
                .execute(
                    ClientRequest::Read {
                        function_code: &function_code,
                        start,
                        count,
                    },
                    unit_id,
                    timeout_ms,
                )
                .await;
            match result {
                Ok(values) => {
                    validate_read_response(&values, count)?;
                    for (offset, value) in values.iter().copied().enumerate() {
                        memory.set_value(&function_code, start + offset as u16, value)?;
                    }
                    return Ok((0..count)
                        .map(|offset| (start + offset, values[offset as usize]))
                        .collect());
                }
                Err(err) => last_error = err,
            }
        }
        return Err(format!(
            "Modbus read failed after {} attempt(s): {last_error}",
            u16::from(retries) + 1
        ));
    }

    memory.get_range(&function_code, start, count)
}

#[tauri::command]
async fn write_register_native(
    session_id: String,
    address: u16,
    value: u16,
    function_code: String,
    unit_id: u8,
    timeout_ms: u32,
    retries: u8,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let operation = connection_operation(&state, &session_id)?;
    let _operation = operation.lock().await;
    validate_request(&function_code, address, 1)?;
    validate_communication_settings(timeout_ms, retries)?;
    let memory = connected_memory(&state, &session_id)?;

    if let Some(client) = cached_client(&state, &session_id)? {
        if client.closing.load(Ordering::SeqCst) {
            return Err("Modbus connection is closing".into());
        }
        let mut last_error = String::new();
        for _ in 0..=retries {
            let result = client
                .execute(
                    ClientRequest::Write {
                        function_code: &function_code,
                        address,
                        value,
                    },
                    unit_id,
                    timeout_ms,
                )
                .await;
            match result {
                Ok(_) => {
                    memory.set_value(&function_code, address, value)?;
                    return Ok(());
                }
                Err(err) => last_error = err,
            }
        }
        return Err(format!(
            "Modbus write failed after {} attempt(s): {last_error}",
            u16::from(retries) + 1
        ));
    }

    memory.set_value(&function_code, address, value)
}

#[tauri::command]
async fn write_registers_native(
    session_id: String,
    start: u16,
    values: Vec<u16>,
    function_code: String,
    unit_id: u8,
    timeout_ms: u32,
    retries: u8,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let operation = connection_operation(&state, &session_id)?;
    let _operation = operation.lock().await;
    let memory = connected_memory(&state, &session_id)?;
    if values.is_empty() {
        return Ok(());
    }
    validate_multiple_write(&function_code, start, values.len())?;
    validate_communication_settings(timeout_ms, retries)?;

    if let Some(client) = cached_client(&state, &session_id)? {
        if client.closing.load(Ordering::SeqCst) {
            return Err("Modbus connection is closing".into());
        }
        let mut last_error = String::new();
        for _ in 0..=retries {
            let result = client
                .execute(
                    ClientRequest::WriteMultiple {
                        function_code: &function_code,
                        start,
                        values: &values,
                    },
                    unit_id,
                    timeout_ms,
                )
                .await;
            match result {
                Ok(_) => {
                    for (offset, &val) in values.iter().enumerate() {
                        memory.set_value(&function_code, start + offset as u16, val)?;
                    }
                    return Ok(());
                }
                Err(err) => last_error = err,
            }
        }
        return Err(format!(
            "Modbus write failed after {} attempt(s): {last_error}",
            u16::from(retries) + 1
        ));
    }

    for (offset, &val) in values.iter().enumerate() {
        memory.set_value(&function_code, start + offset as u16, val)?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            is_portable_installation,
            list_serial_ports_native,
            list_tcp_clients_native,
            connect_modbus_native,
            disconnect_modbus_native,
            read_registers_native,
            write_register_native,
            write_registers_native
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn is_portable_installation() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            if dir.join("uninstall.exe").exists() {
                return false;
            }
        }
    }
    true
}

#[tauri::command]
async fn list_tcp_clients_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<modbus::server::TcpClientInfo>, String> {
    Ok(state.tcp_slaves.lock().await.clients(&session_id))
}

#[tauri::command]
async fn list_serial_ports_native() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(|| {
        let mut ports: Vec<String> = tokio_serial::available_ports()
            .map_err(|err| format!("Failed to enumerate serial ports: {err}"))?
            .into_iter()
            .map(|port| port.port_name)
            .filter(|name| !name.is_empty())
            .collect();
        ports.sort();
        ports.dedup();
        Ok(ports)
    })
    .await
    .map_err(|err| format!("Serial port discovery failed: {err}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use modbus::server::{start_tcp_slave, TcpSlaveRoute, TcpSlaveTask};
    use tokio::net::TcpListener;

    fn tcp_test_config(address: SocketAddr) -> ConnectionConfig {
        serde_json::from_value(serde_json::json!({
            "role": "Master", "protocol": "TCP", "ip": address.ip().to_string(),
            "port": address.port(), "serialPort": "", "baudRate": 9600,
            "dataBits": 8, "stopBits": 1, "parity": "None"
        }))
        .unwrap()
    }

    fn test_response(request: &[u8; 12], value: u16) -> Vec<u8> {
        let mut response = request[..7].to_vec();
        response[4] = 0;
        response[5] = 5;
        response.extend_from_slice(&[3, 2]);
        response.extend_from_slice(&value.to_be_bytes());
        response
    }

    async fn verify_tcp_recovery(wrong_header: bool) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut old, _) = listener.accept().await.unwrap();
            let mut request = [0; 12];
            old.read_exact(&mut request).await.unwrap();
            if wrong_header {
                let mut response = test_response(&request, 999);
                response[1] = response[1].wrapping_add(1);
                old.write_all(&response).await.unwrap();
            }
            // A new socket must be accepted before any successful response is sent.
            let (mut fresh, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
                .await
                .unwrap()
                .unwrap();
            if !wrong_header {
                // Simulate the delayed old response; it must never supply the next read.
                let _ = old.write_all(&test_response(&request, 999)).await;
            }
            fresh.read_exact(&mut request).await.unwrap();
            fresh.write_all(&test_response(&request, 42)).await.unwrap();
        });
        let config = tcp_test_config(address);
        let context = connect_tcp(&config.ip, config.port, 2, 1000, 0)
            .await
            .unwrap();
        let client = ClientConnection::new(context, config);
        let first = client
            .execute(
                ClientRequest::Read {
                    function_code: "0x03",
                    start: 0,
                    count: 1,
                },
                2,
                100,
            )
            .await;
        let error = first.unwrap_err();
        assert!(
            error.contains(if wrong_header {
                "Invalid response header"
            } else {
                "timed out"
            }),
            "{error}"
        );
        assert!(
            client.context.lock().await.is_none(),
            "Failed socket must be dropped even when no retries remain"
        );
        let values = client
            .execute(
                ClientRequest::Read {
                    function_code: "0x03",
                    start: 0,
                    count: 1,
                },
                2,
                1000,
            )
            .await
            .unwrap();
        assert_eq!(values, vec![42]);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn tcp_timeout_discards_late_reply_before_next_request() {
        verify_tcp_recovery(false).await;
    }

    #[tokio::test]
    async fn tcp_header_mismatch_reconnects_before_next_request() {
        verify_tcp_recovery(true).await;
    }

    #[tokio::test]
    async fn healthy_tcp_requests_reuse_one_connection() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            for value in [41, 42] {
                let mut request = [0; 12];
                socket.read_exact(&mut request).await.unwrap();
                socket
                    .write_all(&test_response(&request, value))
                    .await
                    .unwrap();
            }
        });
        let config = tcp_test_config(address);
        let context = connect_tcp(&config.ip, config.port, 2, 1000, 0)
            .await
            .unwrap();
        let client = ClientConnection::new(context, config);
        for value in [41, 42] {
            assert_eq!(
                client
                    .execute(
                        ClientRequest::Read {
                            function_code: "0x03",
                            start: 0,
                            count: 1
                        },
                        2,
                        1000
                    )
                    .await
                    .unwrap(),
                vec![value]
            );
        }
        server.await.unwrap();
    }

    fn start_test_slave(listener: TcpListener, memory: SessionMemory) -> TcpSlaveTask {
        let task = start_tcp_slave(listener);
        task.register(
            1,
            TcpSlaveRoute {
                session_id: "test".into(),
                session_title: "Test".into(),
                memory,
            },
        )
        .unwrap();
        task
    }

    #[test]
    fn valid_range_can_include_the_last_register() {
        assert!(validate_request("0x03", 65_535, 1).is_ok());
        assert!(validate_request("0x03", 65_535, 2).is_err());
    }

    #[test]
    fn disconnected_sessions_cannot_fall_back_to_local_memory() {
        let state = AppState::default();
        let memory = session_memory(&state, "A").unwrap();
        memory.set_value("0x03", 10, 42).unwrap();
        assert!(connected_memory(&state, "A").is_err());
        state
            .connection_status
            .lock()
            .unwrap()
            .insert("A".into(), "Listening".into());
        assert_eq!(
            connected_memory(&state, "A")
                .unwrap()
                .get_range("0x03", 10, 1)
                .unwrap()[&10],
            42
        );
        state.connection_status.lock().unwrap().remove("A");
        assert!(connected_memory(&state, "A").is_err());
    }

    #[test]
    fn read_response_length_must_match_request() {
        assert!(validate_read_response(&[1, 2], 2).is_ok());
        assert!(validate_read_response(&[], 2).is_err());
        assert!(validate_read_response(&[1], 2).is_err());
        assert!(validate_read_response(&[1, 2, 3], 2).is_err());
    }

    #[test]
    fn shared_serial_port_requires_matching_transport_settings() {
        let config: ConnectionConfig = serde_json::from_value(serde_json::json!({
            "role": "Slave", "protocol": "RTU", "ip": "0.0.0.0", "port": 502,
            "serialPort": "COM4", "baudRate": 9600, "dataBits": 8,
            "stopBits": 1, "parity": "None"
        }))
        .unwrap();
        let mut other = config.clone();
        other.timeout_ms = 2000;
        assert!(config.serial_settings_match(&other));
        for field in ["baudRate", "dataBits", "stopBits", "parity"] {
            let mut value = serde_json::to_value(&config).unwrap();
            value[field] = match field {
                "baudRate" => serde_json::json!(19200),
                "dataBits" => serde_json::json!(7),
                "stopBits" => serde_json::json!(2),
                _ => serde_json::json!("Even"),
            };
            assert!(
                !config.serial_settings_match(&serde_json::from_value(value).unwrap()),
                "{field}"
            );
        }
    }

    #[test]
    fn session_memory_isolated_by_data_area() {
        let memory = SessionMemory::new();
        memory.set_value("0x03", 10, 42).unwrap();
        memory.set_value("0x01", 10, 1).unwrap();

        assert_eq!(memory.get_range("0x03", 10, 1).unwrap()[&10], 42);
        assert_eq!(memory.get_range("0x01", 10, 1).unwrap()[&10], 1);
        assert_eq!(memory.get_range("0x04", 10, 1).unwrap()[&10], 0);
    }

    #[tokio::test]
    async fn tcp_slave_serves_coils_and_holding_registers() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = start_test_slave(listener, SessionMemory::new());

        let mut client = tokio_modbus::client::tcp::connect_slave(address, Slave(1))
            .await
            .unwrap();
        client.write_single_coil(7, true).await.unwrap();
        client.write_single_register(8, 0x1234).await.unwrap();

        assert_eq!(client.read_coils(7, 1).await.unwrap(), vec![true]);
        assert_eq!(
            client.read_holding_registers(8, 1).await.unwrap(),
            vec![0x1234]
        );

        task.stop().await;
    }

    #[tokio::test]
    async fn stopped_tcp_slave_releases_its_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = start_test_slave(listener, SessionMemory::new());

        task.stop().await;

        let rebound = TcpListener::bind(address).await;
        assert!(
            rebound.is_ok(),
            "TCP Slave port was not released after stop"
        );
    }

    #[tokio::test]
    async fn stopped_tcp_slave_closes_existing_clients() {
        use tokio::io::AsyncReadExt;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let memory = SessionMemory::new();
        let task = start_test_slave(listener, memory.clone());
        let mut first = tokio_modbus::client::tcp::connect_slave(address, Slave(1))
            .await
            .unwrap();
        let mut second = tokio_modbus::client::tcp::connect_slave(address, Slave(1))
            .await
            .unwrap();
        let mut idle = tokio::net::TcpStream::connect(address).await.unwrap();
        first.write_single_register(8, 42).await.unwrap();
        second.read_holding_registers(8, 1).await.unwrap();

        task.stop().await;

        let write =
            tokio::time::timeout(Duration::from_secs(1), first.write_single_register(8, 99))
                .await
                .expect("Stopped client should be closed promptly");
        assert!(write.is_err());
        let read =
            tokio::time::timeout(Duration::from_secs(1), second.read_holding_registers(8, 1))
                .await
                .expect("Stopped client should be closed promptly");
        assert!(read.is_err());
        let mut byte = [0];
        let idle_read = tokio::time::timeout(Duration::from_secs(1), idle.read(&mut byte))
            .await
            .expect("Idle connections must also close");
        assert!(matches!(idle_read, Ok(0) | Err(_)));
        assert_eq!(memory.get_holding_range(8, 1).unwrap(), vec![42]);

        let listener = TcpListener::bind(address).await.unwrap();
        let restarted = start_test_slave(listener, memory.clone());
        let mut client = tokio_modbus::client::tcp::connect_slave(address, Slave(1))
            .await
            .unwrap();
        client.write_single_register(8, 7).await.unwrap();
        assert_eq!(memory.get_holding_range(8, 1).unwrap(), vec![7]);
        restarted.stop().await;
    }

    #[tokio::test]
    async fn remote_multiple_writes_respect_data_areas() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let memory = SessionMemory::new();
        memory.set_value("0x02", 10, 1).unwrap();
        memory.set_value("0x04", 10, 55).unwrap();
        let task = start_test_slave(listener, memory.clone());
        let mut client = tokio_modbus::client::tcp::connect_slave(address, Slave(1))
            .await
            .unwrap();

        write_remote_multiple(&mut client, "0x03", 10, &[42, 43], 1)
            .await
            .unwrap();
        write_remote_multiple(&mut client, "0x01", 10, &[1, 0, 2], 1)
            .await
            .unwrap();
        for function_code in ["0x02", "0x04"] {
            assert!(
                write_remote_multiple(&mut client, function_code, 10, &[99, 99], 1)
                    .await
                    .unwrap_err()
                    .contains("read-only")
            );
        }
        assert_eq!(
            client.read_holding_registers(10, 2).await.unwrap(),
            vec![42, 43]
        );
        assert_eq!(
            client.read_coils(10, 3).await.unwrap(),
            vec![true, false, true]
        );
        assert_eq!(
            client.read_discrete_inputs(10, 2).await.unwrap(),
            vec![true, false]
        );
        assert_eq!(
            client.read_input_registers(10, 2).await.unwrap(),
            vec![55, 0]
        );
        task.stop().await;
    }

    #[test]
    fn multiple_write_limits_do_not_truncate_large_counts() {
        assert!(validate_multiple_write("0x03", 0, 123).is_ok());
        assert!(validate_multiple_write("0x03", 0, 124).is_err());
        assert!(validate_multiple_write("0x01", 0, 1968).is_ok());
        assert!(validate_multiple_write("0x01", 0, 1969).is_err());
        assert!(validate_multiple_write("0x03", 0, 65_537).is_err());
        assert!(validate_multiple_write("0x03", 65_535, 2).is_err());
        assert!(validate_multiple_write("0x03", 65_535, 1).is_ok());
    }
}
