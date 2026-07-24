# Backend Task 1 Brief: Rust Modbus Backend & Tauri Commands

## Environment & Constraints
- Language: Rust 2021
- Framework: Tauri v2
- Target: `src-tauri/src/modbus.rs` and `src-tauri/src/lib.rs`

## Instructions
1. Create `src-tauri/src/modbus.rs` defining `ConnectionConfig`, `TrafficLog`, `ModbusState`, and Tauri command handlers: `connect_modbus`, `disconnect_modbus`, `read_registers`, `write_register`.
2. Register the module and commands in `src-tauri/src/lib.rs`.

Code for `src-tauri/src/modbus.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub role: String,
    pub protocol: String,
    pub ip: String,
    pub port: u16,
    #[serde(rename = "serialPort")]
    pub serial_port: String,
    #[serde(rename = "baudRate")]
    pub baud_rate: u32,
    #[serde(rename = "dataBits")]
    pub data_bits: u8,
    #[serde(rename = "stopBits")]
    pub stop_bits: u8,
    pub parity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficLog {
    pub id: String,
    pub time: String,
    pub direction: String,
    pub message: String,
    pub bytes: String,
}

pub struct ModbusState {
    pub connected: bool,
    pub registers: HashMap<u16, u16>,
    pub logs: Vec<TrafficLog>,
}

impl Default for ModbusState {
    fn default() -> Self {
        Self {
            connected: false,
            registers: HashMap::new(),
            logs: Vec::new(),
        }
    }
}

pub type SafeState = Mutex<ModbusState>;

#[tauri::command]
pub fn connect_modbus(
    config: ConnectionConfig,
    state: tauri::State<'_, SafeState>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.connected = true;
    let target = if config.protocol == "TCP" {
        format!("{}:{}", config.ip, config.port)
    } else {
        format!("{} ({})", config.serial_port, config.baud_rate)
    };
    Ok(format!("Connected to {}", target))
}

#[tauri::command]
pub fn disconnect_modbus(state: tauri::State<'_, SafeState>) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.connected = false;
    Ok("Disconnected".into())
}

#[tauri::command]
pub fn read_registers(
    start: u16,
    count: u16,
    state: tauri::State<'_, SafeState>,
) -> Result<HashMap<u16, u16>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for addr in start..(start + count) {
        let val = *s.registers.get(&addr).unwrap_or(&0);
        map.insert(addr, val);
    }
    Ok(map)
}

#[tauri::command]
pub fn write_register(
    address: u16,
    value: u16,
    state: tauri::State<'_, SafeState>,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.registers.insert(address, value);
    Ok(())
}
```

Code for `src-tauri/src/lib.rs`:
```rust
mod modbus;

use std::sync::Mutex;
use modbus::{connect_modbus, disconnect_modbus, read_registers, write_register, ModbusState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(ModbusState::default()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            connect_modbus,
            disconnect_modbus,
            read_registers,
            write_register
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/backend-task-1-report.md`.
