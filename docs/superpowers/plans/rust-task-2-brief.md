# Rust Task 2 Brief: Isolated Session Memory & Modbus Server Engine (`server.rs`)

## Environment & Constraints
- Language: Rust 2021
- Rule: "不需要共享，每个会话/寄存器单独一块独立内存" (Each session maintains its own isolated memory space, non-shared across tabs).

## Instructions
1. Create `src-tauri/src/modbus/server.rs` defining `SessionMemory` providing isolated `holding_registers` (`HashMap<u16, u16>`) per session.
2. Implement methods for `get_register`, `set_register`, and `get_range`.

Code for `src-tauri/src/modbus/server.rs`:
```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct SessionMemory {
    pub holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl SessionMemory {
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

    pub fn get_range(&self, start: u16, count: u16) -> HashMap<u16, u16> {
        let map = self.holding_registers.lock().unwrap();
        let mut res = HashMap::new();
        for addr in start..(start + count) {
            let val = *map.get(&addr).unwrap_or(&0);
            res.insert(addr, val);
        }
        res
    }
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/rust-task-2-report.md`.
