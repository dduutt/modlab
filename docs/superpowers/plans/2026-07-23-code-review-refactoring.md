# Code Review & Architecture Optimization Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clean up compiler warnings in Rust backend, optimize multi-session background polling in Vue frontend, add register JSON export button to toolbar, and implement strict input bounds validation for address and count.

**Architecture:** 
1. **Rust Backend Clean-up**: Annotate or utilize `ModbusRole`, `ModbusProtocol`, and `TrafficFrame` in `types.rs` to achieve 0 compiler warnings in `cargo check`.
2. **Multi-Session Timer Management (`App.vue`)**: Refactor tab interval timers from a single active-tab timer into a session-keyed timer map (`sessionTimers: Map<string, ReturnType<typeof setInterval>>`), allowing background polling for ALL connected tabs simultaneously.
3. **Toolbar Export & Validation**: Add an "Export" button to `Toolbar.vue` for saving session register maps to JSON, and enforce min/max constraints on `ConfigRow.vue` input fields.

**Tech Stack:** Rust (Tauri v2), Vue 3 (`<script setup lang="ts">`), TypeScript, TailwindCSS.

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Zero compiler warnings on `cargo check`.
- Zero build errors on `pnpm build`.

---

### Task 1: Rust Backend Warning Cleanup & Event Emission (`types.rs` & `lib.rs`)

**Files:**
- Modify: `src-tauri/src/modbus/types.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: Warning-free Rust backend compilation with Serde annotations.

- [ ] **Step 1: Clean Up Dead Code Warnings in `src-tauri/src/modbus/types.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ModbusRole {
    Slave,
    Master,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum ModbusProtocol {
    Tcp,
    Rtu,
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

- [ ] **Step 2: Add `#[allow(dead_code)]` to unused types in `types.rs`**

```rust
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusRole {
    Slave,
    Master,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusProtocol {
    Tcp,
    Rtu,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficFrame {
    pub id: String,
    pub time: String,
    pub direction: String,
    pub message: String,
    pub bytes: String,
}
```

- [ ] **Step 3: Run `cargo check` to verify 0 warnings**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: `Finished dev profile [unoptimized + debuginfo] target(s) in ...` with ZERO warnings!

- [ ] **Step 4: Commit**

```bash
git add src-tauri/
git commit -m "fix(backend): clean up dead code warnings in Rust types"
```

---

### Task 2: Multi-Session Concurrent Background Polling (`App.vue`)

**Files:**
- Modify: `src/App.vue`

**Interfaces:**
- Produces: Multi-timer map allowing all connected sessions to poll in background simultaneously.

- [ ] **Step 1: Update `src/App.vue` Session Polling Timers**

Refactor timer management in `src/App.vue`:
```typescript
const sessionTimers = new Map<string, ReturnType<typeof setInterval>>();

function startSessionPolling(tab: SessionTab) {
  stopSessionPolling(tab.id);
  const timer = setInterval(async () => {
    if (!tab.connected) return;
    const time = new Date().toLocaleTimeString();
    const unit = tab.config.unitId.toString(16).padStart(2, '0');
    const start = tab.config.startAddress.toString(16).padStart(4, '0');
    const count = tab.config.count.toString(16).padStart(4, '0');

    // Add TX Log
    tab.logs.unshift({
      id: Math.random().toString(36).slice(2),
      time,
      direction: 'TX',
      message: `Read Holding (0x03) Req`,
      bytes: `${unit} 03 ${start.slice(0, 2)} ${start.slice(2)} ${count.slice(0, 2)} ${count.slice(2)} C5 D3`,
    });

    try {
      const fetched = await ModbusService.readRegisters(tab.id, tab.config.startAddress, tab.config.count);
      if (Object.keys(fetched).length > 0) {
        tab.values = { ...tab.values, ...fetched };
      }
    } catch (err) {
      console.error(`Polling failed for ${tab.id}:`, err);
    }

    // Add RX Log
    tab.logs.unshift({
      id: Math.random().toString(36).slice(2),
      time,
      direction: 'RX',
      message: `Read Holding (0x03) Resp`,
      bytes: `${unit} 03 50 ${Array.from({ length: 8 }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, '0')).join(' ')} ...`,
    });

    if (tab.logs.length > 100) {
      tab.logs.pop();
    }

    if (tab.config.raw) {
      const addr = tab.config.startAddress + Math.floor(Math.random() * tab.config.count);
      const val = Math.floor(Math.random() * 65535);
      tab.values[addr] = val;
      await ModbusService.writeRegister(tab.id, addr, val);
    }
  }, tab.config.interval || 1000);

  sessionTimers.set(tab.id, timer);
}

function stopSessionPolling(sessionId: string) {
  if (sessionTimers.has(sessionId)) {
    clearInterval(sessionTimers.get(sessionId)!);
    sessionTimers.delete(sessionId);
  }
}
```

- [ ] **Step 2: Connect `startSessionPolling` and `stopSessionPolling` in Connect/Disconnect handlers**

- [ ] **Step 3: Run `pnpm build` to verify compilation**

Run: `pnpm build`
Expected: PASS cleanly.

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat(frontend): support concurrent background polling across multiple sessions"
```

---

### Task 3: Export Functionality & Input Constraints (`Toolbar.vue` & `ConfigRow.vue`)

**Files:**
- Modify: `src/components/Toolbar.vue`
- Modify: `src/components/ConfigRow.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Props: `onExport` emit in `Toolbar.vue`.
- Input validation: `min="0" max="65535"` on Start Address, `min="1" max="2000"` on Count in `ConfigRow.vue`.

- [ ] **Step 1: Add Export Button to `Toolbar.vue`**

```vue
<script setup lang="ts">
import { Settings, Play, Square, Shuffle, TrendingUp, Download } from '@lucide/vue';

defineProps<{
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connect'): void;
  (e: 'open-settings'): void;
  (e: 'fill-random'): void;
  (e: 'fill-increment'): void;
  (e: 'export-json'): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-3 bg-white border-b border-gray-200">
    <div class="flex items-center gap-4">
      <span class="font-semibold text-gray-900 text-base">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-sm text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <button
        @click="emit('toggle-connect')"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium border transition cursor-pointer',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100'
            : 'bg-white text-gray-900 border-gray-300 hover:bg-gray-50'
        ]"
      >
        <component :is="connected ? Square : Play" class="w-4 h-4" />
        {{ connected ? 'Disconnect' : 'Connect' }}
      </button>

      <button
        @click="emit('open-settings')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-white text-gray-700 border border-gray-200 hover:bg-gray-50 transition cursor-pointer"
      >
        <Settings class="w-4 h-4 text-gray-500" />
        Settings
      </button>
    </div>

    <div class="flex items-center gap-3">
      <button
        @click="emit('export-json')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer"
        title="Export Session to JSON"
      >
        <Download class="w-3.5 h-3.5 text-gray-500" />
        Export
      </button>
      <button
        @click="emit('fill-random')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer"
      >
        <Shuffle class="w-3.5 h-3.5 text-gray-500" />
        Random
      </button>
      <button
        @click="emit('fill-increment')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer"
      >
        <TrendingUp class="w-3.5 h-3.5 text-gray-500" />
        Increment
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Add Min/Max Constraints in `ConfigRow.vue`**

Add `min="0" max="65535"` on Start Address and `min="1" max="2000"` on Count input fields.

- [ ] **Step 3: Wire `exportRegistersToJSON` in `App.vue`**

Import `exportRegistersToJSON` from `./utils/export` and bind to `@export-json` event.

- [ ] **Step 4: Verify Project Compilation**

Run: `pnpm build`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add export to JSON and input field validation constraints"
```
