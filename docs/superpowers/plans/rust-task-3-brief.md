# Rust Task 3 Brief: Session-Isolated Tauri Commands (`lib.rs`)

## Environment & Constraints
- Language: Rust 2021
- Rule: "每个会话 (session_id) 拥有完全独立隔离的内存区域"

## Instructions
Update `src-tauri/src/lib.rs` to maintain `HashMap<String, SessionMemory>` in `AppState`, and accept `sessionId: String` parameter in `connect_modbus_native`, `disconnect_modbus_native`, `read_registers_native`, and `write_register_native`.

Code for `src-tauri/src/lib.rs`:
```rust
mod modbus;

use std::collections::HashMap;
use std::sync::Mutex;
use modbus::types::ConnectionConfig;
use modbus::server::SessionMemory;

pub struct AppState {
    pub sessions: Mutex<HashMap<String, SessionMemory>>,
    pub connection_status: Mutex<HashMap<String, String>>,
}

#[tauri::command]
pub async fn connect_modbus_native(
    session_id: String,
    config: ConnectionConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let target = if config.protocol == "TCP" {
        format!("{}:{}", config.ip, config.port)
    } else {
        format!("{} ({})", config.serial_port, config.baud_rate)
    };

    // Initialize isolated memory for this session if not exists
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.entry(session_id.clone()).or_insert_with(SessionMemory::new);
    }

    {
        let mut status = state.connection_status.lock().unwrap();
        status.insert(session_id, target.clone());
    }

    Ok(format!("Connected to {}", target))
}

#[tauri::command]
pub async fn disconnect_modbus_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut status = state.connection_status.lock().unwrap();
    status.remove(&session_id);
    Ok("Disconnected".into())
}

#[tauri::command]
pub async fn read_registers_native(
    session_id: String,
    start: u16,
    count: u16,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    let sessions = state.sessions.lock().unwrap();
    if let Some(mem) = sessions.get(&session_id) {
        Ok(mem.get_range(start, count))
    } else {
        // Return empty range if session not yet created
        let mem = SessionMemory::new();
        Ok(mem.get_range(start, count))
    }
}

#[tauri::command]
pub async fn write_register_native(
    session_id: String,
    address: u16,
    value: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().unwrap();
    let mem = sessions.entry(session_id).or_insert_with(SessionMemory::new);
    mem.set_register(address, value);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            sessions: Mutex::new(HashMap::new()),
            connection_status: Mutex::new(HashMap::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            connect_modbus_native,
            disconnect_modbus_native,
            read_registers_native,
            write_register_native
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/rust-task-3-report.md`.
