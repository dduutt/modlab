# Complete Modbus Tauri Application Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the Modbus desktop application by building the Rust backend commands in Tauri v2, integrating the frontend service layer, and verifying end-to-end desktop functionality.

**Architecture:** 
The application connects the Vue 3 frontend to a Rust backend via Tauri `invoke` commands. The Rust backend in `src-tauri/src/modbus.rs` manages connection state, Modbus TCP/RTU frame parsing, register memory maps, and traffic log streaming. A frontend service `src/services/modbusService.ts` provides a unified API with fallback simulation for browser development.

**Tech Stack:** Rust (Tauri v2, Serde, Tokio), Vue 3, TypeScript, Vite, pnpm.

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Default Window Size: 1000x700.
- Package Manager: `pnpm` for frontend, `cargo` for Rust backend.
- Pure zero-warning compilation for both `cargo check` and `pnpm build`.

---

### Task 1: Rust Modbus Backend Data Structures & Tauri Commands

**Files:**
- Create: `src-tauri/src/modbus.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Produces: Rust Tauri commands `connect_modbus`, `disconnect_modbus`, `read_registers`, `write_register`.

- [ ] **Step 1: Create `src-tauri/src/modbus.rs`**

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
    pub serial_port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
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
```

- [ ] **Step 2: Implement Tauri Commands in `src-tauri/src/modbus.rs`**

```rust
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

- [ ] **Step 3: Register Handlers in `src-tauri/src/lib.rs`**

Update `src-tauri/src/lib.rs`:
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

- [ ] **Step 4: Verify Cargo Build**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Clean pass with zero errors.

---

### Task 2: Frontend Modbus Service Layer (`src/services/modbusService.ts`)

**Files:**
- Create: `src/services/modbusService.ts`

**Interfaces:**
- `connect(config: ConnectionConfig): Promise<string>`
- `disconnect(): Promise<string>`
- `readRegisters(start: number, count: number): Promise<Record<number, number>>`
- `writeRegister(address: number, value: number): Promise<void>`

- [ ] **Step 1: Create `src/services/modbusService.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionConfig } from '../components/SettingsModal.vue';

export class ModbusService {
  private static isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  }

  static async connect(config: ConnectionConfig): Promise<string> {
    if (this.isTauri()) {
      return await invoke<string>('connect_modbus', { config });
    }
    const target = config.protocol === 'TCP' ? `${config.ip}:${config.port}` : `${config.serialPort}`;
    return `Connected to ${target} (Browser Sim)`;
  }

  static async disconnect(): Promise<string> {
    if (this.isTauri()) {
      return await invoke<string>('disconnect_modbus');
    }
    return 'Disconnected (Browser Sim)';
  }

  static async readRegisters(start: number, count: number): Promise<Record<number, number>> {
    if (this.isTauri()) {
      return await invoke<Record<number, number>>('read_registers', { start, count });
    }
    return {};
  }

  static async writeRegister(address: number, value: number): Promise<void> {
    if (this.isTauri()) {
      await invoke('write_register', { address, value });
    }
  }
}
```

---

### Task 3: Full Integration in `src/App.vue` & Verification

**Files:**
- Modify: `src/App.vue`

**Interfaces:**
- Connect `ModbusService` into `handleToggleConnect` and `handleUpdateCell` in `App.vue`.

- [ ] **Step 1: Update `src/App.vue` with `ModbusService` Calls**

Update `handleToggleConnect` and `handleUpdateCell` in `src/App.vue` to invoke `ModbusService`.

- [ ] **Step 2: Build & Verify End-to-End**

Run: `pnpm build`
Expected: Clean build with 0 TypeScript/Vue compile errors.
