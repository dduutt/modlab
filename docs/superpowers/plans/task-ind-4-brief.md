# Task 4 Brief: Timeout & Retries Parameters in Settings Modal (`SettingsModal.vue` & `types.rs`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Backend: Rust (Tauri v2)

## Instructions
1. Update `src/components/SettingsModal.vue` to add `timeoutMs` (default `1000`) and `retries` (default `3`) fields to `ConnectionConfig`.
2. Update `src-tauri/src/modbus/types.rs` to include `pub timeout_ms: u32` and `pub retries: u8` with `#[serde(rename = "timeoutMs")]` and `#[serde(rename = "retries")]`.
3. Update `src/App.vue` default connection configurations to include `timeoutMs: 1000` and `retries: 3`.

Code for `src/components/SettingsModal.vue`:
```vue
<script setup lang="ts">
import { ref, watch } from 'vue';
import { Settings, X, Server, Cpu, Wifi, Radio } from '@lucide/vue';

export interface ConnectionConfig {
  role: 'Slave' | 'Master';
  protocol: 'TCP' | 'RTU';
  ip: string;
  port: number;
  serialPort: string;
  baudRate: number;
  dataBits: number;
  stopBits: number;
  parity: 'None' | 'Even' | 'Odd';
  timeoutMs: number;
  retries: number;
}

const props = defineProps<{
  show: boolean;
  config: ConnectionConfig;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', newConfig: ConnectionConfig): void;
}>();

const form = ref<ConnectionConfig>({ ...props.config });

watch(
  () => props.config,
  (newVal) => {
    form.value = { ...newVal };
  },
  { deep: true }
);

function handleSave() {
  emit('save', { ...form.value });
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <Settings class="w-5 h-5 text-gray-700" />
          <h3 class="font-semibold text-gray-900 text-base">Connection Settings</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 space-y-4 text-sm max-h-[80vh] overflow-y-auto">
        <!-- Role -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Modbus Role</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.role = 'Slave'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Server class="w-4 h-4" /> Slave (Server)
            </button>
            <button
              type="button"
              @click="form.role = 'Master'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cpu class="w-4 h-4" /> Master (Client)
            </button>
          </div>
        </div>

        <!-- Protocol -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Protocol Type</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.protocol = 'TCP'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.protocol === 'TCP' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Wifi class="w-4 h-4" /> Modbus TCP
            </button>
            <button
              type="button"
              @click="form.protocol = 'RTU'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.protocol === 'RTU' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Radio class="w-4 h-4" /> Modbus RTU (Serial)
            </button>
          </div>
        </div>

        <!-- TCP Options -->
        <div v-if="form.protocol === 'TCP'" class="grid grid-cols-3 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div class="col-span-2">
            <label class="block text-xs font-semibold text-gray-500 mb-1">IP Address</label>
            <input v-model="form.ip" type="text" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Port</label>
            <input v-model.number="form.port" type="number" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
        </div>

        <!-- RTU Options -->
        <div v-else class="grid grid-cols-2 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Serial Port</label>
            <input v-model="form.serialPort" type="text" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Baud Rate</label>
            <select v-model.number="form.baudRate" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option :value="9600">9600</option>
              <option :value="19200">19200</option>
              <option :value="38400">38400</option>
              <option :value="57600">57600</option>
              <option :value="115200">115200</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Data Bits</label>
            <select v-model.number="form.dataBits" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option :value="7">7</option>
              <option :value="8">8</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Parity</label>
            <select v-model="form.parity" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option value="None">None</option>
              <option value="Even">Even</option>
              <option value="Odd">Odd</option>
            </select>
          </div>
        </div>

        <!-- Timeout & Retries -->
        <div class="grid grid-cols-2 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Timeout (ms)</label>
            <input v-model.number="form.timeoutMs" type="number" min="100" max="10000" step="100" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">Retries</label>
            <input v-model.number="form.retries" type="number" min="0" max="10" class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="handleSave" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Save Settings</button>
      </div>
    </div>
  </div>
</template>
```

Code for `src-tauri/src/modbus/types.rs`:
```rust
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "timeoutMs", default = "default_timeout_ms")]
    pub timeout_ms: u32,
    #[serde(rename = "retries", default = "default_retries")]
    pub retries: u8,
}

fn default_timeout_ms() -> u32 {
    1000
}

fn default_retries() -> u8 {
    3
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

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-ind-4-report.md`.
