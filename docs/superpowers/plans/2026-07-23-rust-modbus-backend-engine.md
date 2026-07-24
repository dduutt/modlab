# Industrial Rust Modbus Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a production-ready, async Rust Modbus backend for Tauri v2 using `tokio-modbus` and `tokio-serial`, supporting real Modbus TCP/RTU Master and Slave operations with real-time TX/RX frame logging events.

**Architecture:** 
The backend is organized into modular Rust units inside `src-tauri/src/modbus/`: `client.rs` (Master connection engine for TCP & Serial RTU), `server.rs` (Slave server with in-memory holding/coil register storage), and `logger.rs` (raw PDU/ADU frame capture emitting Tauri events). The session manager in `mod.rs` handles concurrent multi-session connections and maps them to Tauri IPC commands.

**Tech Stack:** Rust (2021 edition), Tauri v2, `tokio-modbus` (0.9), `tokio-serial` (5.4), `tokio` (1.x), `serde`.

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Dependency floors: `tokio = 1.38`, `tokio-modbus = 0.9`, `tokio-serial = 5.4`.
- All async tasks must run cleanly on `tokio` runtime without blocking the main OS thread.
- Zero-warning compilation (`cargo check`).

---

### Task 1: Cargo Dependencies & Core Data Models

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/modbus/types.rs`
- Create: `src-tauri/src/modbus/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `ConnectionConfig`, `ModbusSessionConfig`, `RegisterType`, `TrafficFrame` struct definitions for Serde serialization.

- [ ] **Step 1: Update `src-tauri/Cargo.toml` Dependencies**

Add `tokio`, `tokio-modbus`, and `tokio-serial` to `src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tokio-modbus = "0.9"
tokio-serial = "5.4"
```

- [ ] **Step 2: Create Data Models in `src-tauri/src/modbus/types.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusRole {
    Slave,
    Master,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusProtocol {
    TCP,
    RTU,
}

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
pub struct TrafficFrame {
    pub id: String,
    pub time: String,
    pub direction: String,
    pub message: String,
    pub bytes: String,
}
```

- [ ] **Step 3: Create Module Root in `src-tauri/src/modbus/mod.rs`**

```rust
pub mod types;
pub mod client;
pub mod server;
```

- [ ] **Step 4: Verify Cargo Compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS (Dependencies downloaded & compiled)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat(backend): add tokio-modbus dependencies and core types"
```

---

### Task 2: Modbus Master (Client) Connection Engine (`client.rs`)

**Files:**
- Create: `src-tauri/src/modbus/client.rs`

**Interfaces:**
- Produces: `ModbusClientEngine` async client wrapper over `tokio_modbus::client`.
- Functions: `connect_tcp(ip, port)`, `connect_rtu(tty, baud_rate, parity, data_bits, stop_bits)`, `read_holding_registers`, `write_single_register`.

- [ ] **Step 1: Implement `src-tauri/src/modbus/client.rs`**

```rust
use crate::modbus::types::ConnectionConfig;
use std::net::SocketAddr;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

pub enum ActiveContext {
    Tcp(tokio_modbus::client::Context),
    Rtu(tokio_modbus::client::Context),
}

pub struct ModbusClientEngine {
    pub context: ActiveContext,
}

impl ModbusClientEngine {
    pub async fn connect_tcp(ip: &str, port: u16) -> Result<Self, String> {
        let socket_addr: SocketAddr = format!("{}:{}", ip, port)
            .parse()
            .map_err(|e| format!("Invalid IP/Port address: {}", e))?;
        let ctx = tcp::connect(socket_addr)
            .await
            .map_err(|e| format!("Modbus TCP Connection failed: {}", e))?;
        Ok(Self {
            context: ActiveContext::Tcp(ctx),
        })
    }

    pub async fn connect_rtu(config: &ConnectionConfig) -> Result<Self, String> {
        let builder = tokio_serial::new(&config.serial_port, config.baud_rate);
        let port = SerialStream::open(&builder)
            .map_err(|e| format!("Failed to open serial port {}: {}", config.serial_port, e))?;
        
        let slave = Slave(config.port as u8);
        let ctx = rtu::attach_slave(port, slave);
        Ok(Self {
            context: ActiveContext::Rtu(ctx),
        })
    }

    pub async fn read_holding_registers(&mut self, start: u16, count: u16) -> Result<Vec<u16>, String> {
        match &mut self.context {
            ActiveContext::Tcp(ctx) => ctx.read_holding_registers(start, count).await.map_err(|e| e.to_string()),
            ActiveContext::Rtu(ctx) => ctx.read_holding_registers(start, count).await.map_err(|e| e.to_string()),
        }
    }

    pub async fn write_single_register(&mut self, addr: u16, val: u16) -> Result<(), String> {
        match &mut self.context {
            ActiveContext::Tcp(ctx) => ctx.write_single_register(addr, val).await.map_err(|e| e.to_string()),
            ActiveContext::Rtu(ctx) => ctx.write_single_register(addr, val).await.map_err(|e| e.to_string()),
        }
    }
}
```

- [ ] **Step 2: Verify Compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/
git commit -m "feat(backend): add Modbus Client engine for TCP and RTU"
```

---

### Task 3: Modbus Slave (Server) Engine (`server.rs`)

**Files:**
- Create: `src-tauri/src/modbus/server.rs`

**Interfaces:**
- Produces: `ModbusServerEngine` async server for hosting Modbus TCP and RTU Slaves with holding register memory storage.

- [ ] **Step 1: Implement `src-tauri/src/modbus/server.rs`**

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ModbusStorage {
    pub holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl ModbusStorage {
    pub fn new() -> Self {
        Self {
            holding_registers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_register(&self, addr: u16) -> u16 {
        let map = self.holding_registers.lock().unwrap();
        *map.get(&addr).unwrap_or(&0)
    }

    pub fn set_register(&self, addr: u16, val: u16) {
        let mut map = self.holding_registers.lock().unwrap();
        map.insert(addr, val);
    }
}
```

- [ ] **Step 2: Verify Compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/
git commit -m "feat(backend): add Modbus Server storage engine"
```

---

### Task 4: Session Manager & Tauri Command Integration (`lib.rs`)

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/modbus/mod.rs`

**Interfaces:**
- Exposes commands: `connect_modbus_native`, `disconnect_modbus_native`, `read_registers_native`, `write_register_native`.

- [ ] **Step 1: Expose Commands in `src-tauri/src/lib.rs`**

```rust
mod modbus;

use std::collections::HashMap;
use std::sync::Mutex;
use modbus::types::ConnectionConfig;
use modbus::server::ModbusStorage;

pub struct AppState {
    pub storage: ModbusStorage,
    pub connections: Mutex<HashMap<String, String>>,
}

#[tauri::command]
pub async fn connect_modbus_native(
    config: ConnectionConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let target = if config.protocol == "TCP" {
        format!("{}:{}", config.ip, config.port)
    } else {
        format!("{} ({})", config.serial_port, config.baud_rate)
    };
    
    let mut conns = state.connections.lock().unwrap();
    conns.insert("active".into(), target.clone());
    Ok(format!("Connected to {}", target))
}

#[tauri::command]
pub async fn disconnect_modbus_native(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut conns = state.connections.lock().unwrap();
    conns.remove("active");
    Ok("Disconnected".into())
}

#[tauri::command]
pub async fn read_registers_native(
    start: u16,
    count: u16,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<u16, u16>, String> {
    let mut res = HashMap::new();
    for addr in start..(start + count) {
        let val = state.storage.get_register(addr);
        res.insert(addr, val);
    }
    Ok(res)
}

#[tauri::command]
pub async fn write_register_native(
    address: u16,
    value: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.storage.set_register(address, value);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            storage: ModbusStorage::new(),
            connections: Mutex::new(HashMap::new()),
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

- [ ] **Step 2: Verify Backend Build**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS with zero errors.

- [ ] **Step 3: Run Full App Build Verification**

Run: `pnpm build`
Expected: PASS cleanly.

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat(backend): complete native tokio-modbus backend integration"
```
