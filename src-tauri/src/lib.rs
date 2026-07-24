mod modbus;

use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;
use tokio_modbus::client::Context as ModbusContext;
use tokio_modbus::prelude::*;

use modbus::types::ConnectionConfig;
use modbus::server::SessionMemory;
use modbus::client::{connect_tcp, connect_rtu};

pub struct AppState {
    pub sessions: Mutex<HashMap<String, SessionMemory>>,
    pub connection_status: Mutex<HashMap<String, String>>,
    pub clients: AsyncMutex<HashMap<String, ModbusContext>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            connection_status: Mutex::new(HashMap::new()),
            clients: AsyncMutex::new(HashMap::new()),
        }
    }
}

#[tauri::command]
async fn connect_modbus_native(
    session_id: String,
    config: ConnectionConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let target = if config.protocol == "TCP" {
        format!("{}:{}", config.ip, config.port)
    } else {
        format!("{} ({})", config.serial_port, config.baud_rate)
    };

    // Initialize session memory
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.entry(session_id.clone()).or_insert_with(SessionMemory::new);
    }

    if config.role == "Master" {
        if config.protocol == "TCP" {
            if config.ip == "0.0.0.0" {
                return Err("Invalid target IP '0.0.0.0'. Please enter target Slave IP (e.g. 127.0.0.1)".into());
            }
            match connect_tcp(&config.ip, config.port).await {
                Ok(ctx) => {
                    let mut clients = state.clients.lock().await;
                    clients.insert(session_id.clone(), ctx);
                    let mut status = state.connection_status.lock().unwrap();
                    status.insert(session_id, target.clone());
                    return Ok(format!("Connected to Modbus TCP ({})", target));
                }
                Err(err) => {
                    return Err(format!("TCP Connect Error: {}", err));
                }
            }
        } else if config.protocol == "RTU" {
            match connect_rtu(&config.serial_port, config.baud_rate).await {
                Ok(ctx) => {
                    let mut clients = state.clients.lock().await;
                    clients.insert(session_id.clone(), ctx);
                    let mut status = state.connection_status.lock().unwrap();
                    status.insert(session_id, target.clone());
                    return Ok(format!("Connected to Modbus RTU ({})", target));
                }
                Err(err) => {
                    return Err(format!("RTU Connect Error: {}", err));
                }
            }
        }
    }

    {
        let mut status = state.connection_status.lock().unwrap();
        status.insert(session_id, target.clone());
    }

    Ok(format!("Slave Listening on {}", target))
}

#[tauri::command]
async fn disconnect_modbus_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    {
        let mut clients = state.clients.lock().await;
        clients.remove(&session_id);
    }
    let mut status = state.connection_status.lock().unwrap();
    status.remove(&session_id);
    Ok("Disconnected".into())
}

#[tauri::command]
async fn read_registers_native(
    session_id: String,
    start: u16,
    count: u16,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    // 1. Try real tokio-modbus TCP client read first
    {
        let mut clients = state.clients.lock().await;
        if let Some(ctx) = clients.get_mut(&session_id) {
          match ctx.read_holding_registers(start, count).await {
            Ok(vec) => {
              let mut map = HashMap::new();
              for (idx, val) in vec.into_iter().enumerate() {
                map.insert(start + idx as u16, val);
              }
              if let Ok(mut sessions) = state.sessions.lock() {
                if let Some(mem) = sessions.get_mut(&session_id) {
                  for (&addr, &val) in &map {
                    mem.set_register(addr, val);
                  }
                }
              }
              return Ok(map);
            }
            Err(err) => {
              eprintln!("[tokio-modbus] Read error: {}", err);
            }
          }
        }
    }

    // 2. Local session memory fallback
    let sessions = state.sessions.lock().unwrap();
    if let Some(mem) = sessions.get(&session_id) {
        Ok(mem.get_range(start, count))
    } else {
        let mem = SessionMemory::new();
        Ok(mem.get_range(start, count))
    }
}

#[tauri::command]
async fn write_register_native(
    session_id: String,
    address: u16,
    value: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1. Write to tokio-modbus TCP client if connected
    {
        let mut clients = state.clients.lock().await;
        if let Some(ctx) = clients.get_mut(&session_id) {
            if let Err(err) = ctx.write_single_register(address, value).await {
                eprintln!("[tokio-modbus] Write error: {}", err);
            }
        }
    }

    // 2. Sync local session memory
    let mut sessions = state.sessions.lock().unwrap();
    let mem = sessions.entry(session_id).or_insert_with(SessionMemory::new);
    mem.set_register(address, value);
    Ok(())
}

#[tauri::command]
async fn fill_random_native(
    session_id: String,
    start: u16,
    count: u16,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    let mut sessions = state.sessions.lock().unwrap();
    let mem = sessions.entry(session_id).or_insert_with(SessionMemory::new);
    let mut res = HashMap::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for (idx, addr) in (start..(start + count)).enumerate() {
        let val = ((now.wrapping_add((idx as u128) * 1103515245 + 12345)) % 1000) as u16;
        mem.set_register(addr, val);
        res.insert(addr, val);
    }
    Ok(res)
}

#[tauri::command]
async fn fill_increment_native(
    session_id: String,
    start: u16,
    count: u16,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    let mut sessions = state.sessions.lock().unwrap();
    let mem = sessions.entry(session_id).or_insert_with(SessionMemory::new);
    let mut res = HashMap::new();
    let mut val: u16 = 1;
    for addr in start..(start + count) {
        mem.set_register(addr, val);
        res.insert(addr, val);
        val = val.wrapping_add(1);
    }
    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            connect_modbus_native,
            disconnect_modbus_native,
            read_registers_native,
            write_register_native,
            fill_random_native,
            fill_increment_native
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_session_isolation() {
        let state = AppState::default();

        {
            let mut sessions = state.sessions.lock().unwrap();
            let mem_a = sessions.entry("session-A".to_string()).or_insert_with(SessionMemory::new);
            mem_a.set_register(100, 42);
        }

        {
            let mut sessions = state.sessions.lock().unwrap();
            let mem_b = sessions.entry("session-B".to_string()).or_insert_with(SessionMemory::new);
            mem_b.set_register(100, 99);
        }

        {
            let sessions = state.sessions.lock().unwrap();
            let mem_a = sessions.get("session-A").unwrap();
            let mem_b = sessions.get("session-B").unwrap();

            assert_eq!(mem_a.get_register(100), 42);
            assert_eq!(mem_b.get_register(100), 99);
        }
    }
}
