# Rust Task 1 Brief: Cargo Dependencies & Core Data Models

## Environment & Constraints
- Language: Rust 2021
- Package Manager: `cargo`
- Target: `src-tauri/Cargo.toml`, `src-tauri/src/modbus/types.rs`, `src-tauri/src/modbus/mod.rs`

## Instructions
1. Add `tokio`, `tokio-modbus`, `tokio-serial` to `src-tauri/Cargo.toml`.
2. Create `src-tauri/src/modbus/types.rs` with `ConnectionConfig`, `ModbusRole`, `ModbusProtocol`, and `TrafficFrame`.
3. Create `src-tauri/src/modbus/mod.rs`.

Code for `src-tauri/Cargo.toml`:
```toml
[package]
name = "modlab"
version = "0.1.0"
description = "A Tauri App"
authors = ["you"]
edition = "2021"

[lib]
name = "modlab_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tokio-modbus = "0.9"
tokio-serial = "5.4"
```

Code for `src-tauri/src/modbus/types.rs`:
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

Code for `src-tauri/src/modbus/mod.rs`:
```rust
pub mod types;
pub mod client;
pub mod server;
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/rust-task-1-report.md`.
