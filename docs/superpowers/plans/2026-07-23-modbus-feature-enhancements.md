# Modbus App Feature Enhancements Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enhance the Modbus desktop application with full functional capabilities: Settings modal (TCP/RTU, Master/Slave), Data Format conversion (Dec/Hex/Bin, Int16/UInt16/Int32/Float32), Traffic/Communication Log Drawer (TX/RX Hex stream), Tab Renaming, Export/Import register maps, and Simulated Polling/Serving engine.

**Architecture:** 
- **Settings Modal (`SettingsModal.vue`)**: Modal dialog for connection parameters (TCP IP/Port, Serial COM Port, Baudrate, Parity, Data/Stop bits, Mode).
- **Log Panel (`TrafficLog.vue`)**: Collapsible bottom drawer showing real-time Modbus frame logs (TX/RX bytes in hex format).
- **Data Formatter Engine (`src/utils/modbusFormatter.ts`)**: Utility module handling Dec/Hex/Bin conversions and Int16/UInt16/Int32/Float32 register byte packing/unpacking.
- **Export/Import (`src/utils/fileHandler.ts`)**: Export/import register mapping & values via JSON/CSV.
- **Simulated Engine**: Live polling ticker in Vue state when "Connect" is active (simulating read/write & interval ticks).

**Tech Stack:** Vue 3, TypeScript, Lucide Icons (`@lucide/vue`), TailwindCSS.

## Global Constraints
- Target Window Size: 1000x700
- Package Manager: `pnpm`
- Pure Vue 3 + TS implementation with zero build errors (`pnpm build`).

---

### Task 1: Settings Modal (`SettingsModal.vue`)

**Files:**
- Create: `src/components/SettingsModal.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Props: `show: boolean`, `initialConfig: ConnectionConfig`
- Emits: `close`, `save(config: ConnectionConfig)`

- [ ] **Step 1: Create `src/components/SettingsModal.vue`**

```vue
<script setup lang="ts">
import { ref, watch } from 'vue';
import { X, Server, Cpu, Cable, Network } from 'lucide-vue-next';

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
}

const props = defineProps<{
  show: boolean;
  config: ConnectionConfig;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', config: ConnectionConfig): void;
}>();

const form = ref<ConnectionConfig>({ ...props.config });

watch(() => props.config, (newVal) => {
  form.value = { ...newVal };
}, { deep: true });

function handleSave() {
  emit('save', { ...form.value });
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <Settings class="w-5 h-5 text-blue-600" />
          <h3 class="font-semibold text-gray-900 text-base">Connection Settings</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="p-6 space-y-4 text-sm">
        <!-- Mode Switcher -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Mode / Role</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.role = 'Slave'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition', form.role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Server class="w-4 h-4" /> Slave (Server)
            </button>
            <button
              type="button"
              @click="form.role = 'Master'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition', form.role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cpu class="w-4 h-4" /> Master (Client)
            </button>
          </div>
        </div>

        <!-- Protocol Switcher -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Protocol</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.protocol = 'TCP'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition', form.protocol === 'TCP' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Network class="w-4 h-4" /> Modbus TCP
            </button>
            <button
              type="button"
              @click="form.protocol = 'RTU'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition', form.protocol === 'RTU' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cable class="w-4 h-4" /> Modbus RTU (Serial)
            </button>
          </div>
        </div>

        <!-- TCP Fields -->
        <template v-if="form.protocol === 'TCP'">
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2">
              <label class="block text-xs font-semibold text-gray-500 mb-1">IP Address</label>
              <input v-model="form.ip" type="text" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-mono text-sm" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Port</label>
              <input v-model.number="form.port" type="number" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-mono text-sm text-center" />
            </div>
          </div>
        </template>

        <!-- RTU Fields -->
        <template v-else>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Serial Port</label>
              <select v-model="form.serialPort" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white font-mono text-sm">
                <option value="COM1">COM1</option>
                <option value="COM2">COM2</option>
                <option value="COM3">COM3</option>
                <option value="/dev/ttyUSB0">/dev/ttyUSB0</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Baud Rate</label>
              <select v-model.number="form.baudRate" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white font-mono text-sm">
                <option :value="4800">4800</option>
                <option :value="9600">9600</option>
                <option :value="19200">19200</option>
                <option :value="38400">38400</option>
                <option :value="115200">115200</option>
              </select>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Data Bits</label>
              <select v-model.number="form.dataBits" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center">
                <option :value="7">7</option>
                <option :value="8">8</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Stop Bits</label>
              <select v-model.number="form.stopBits" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center">
                <option :value="1">1</option>
                <option :value="2">2</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Parity</label>
              <select v-model="form.parity" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center">
                <option value="None">None</option>
                <option value="Even">Even</option>
                <option value="Odd">Odd</option>
              </select>
            </div>
          </div>
        </template>
      </div>

      <div class="flex items-center justify-end gap-3 px-6 py-4 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-2 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm">Cancel</button>
        <button @click="handleSave" class="px-5 py-2 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm">Save Changes</button>
      </div>
    </div>
  </div>
</template>
```

---

### Task 2: Data Formatting & Number Representation Converter (`src/utils/modbusFormatter.ts` & `DataGrid.vue`)

**Files:**
- Create: `src/utils/modbusFormatter.ts`
- Modify: `src/components/DataGrid.vue`

**Interfaces:**
- `formatValue(val: number, format: 'Dec' | 'Hex' | 'Bin', dataType: string): string`

- [ ] **Step 1: Create `src/utils/modbusFormatter.ts`**

```typescript
export function formatValue(raw: number, format: string, dataType: string): string {
  const num = raw ?? 0;
  if (format === 'Hex') {
    return '0x' + (num & 0xffff).toString(16).toUpperCase().padStart(4, '0');
  }
  if (format === 'Bin') {
    const binStr = (num & 0xffff).toString(2).padStart(16, '0');
    return binStr.replace(/(.{4})/g, '$1 ').trim();
  }
  return num.toString();
}

export function parseFormattedValue(input: string, format: string): number {
  const trimmed = input.trim();
  if (format === 'Hex' || trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return parseInt(trimmed.replace(/^0x/i, ''), 16) || 0;
  }
  if (format === 'Bin') {
    return parseInt(trimmed.replace(/\s+/g, ''), 2) || 0;
  }
  return parseInt(trimmed, 10) || 0;
}
```

- [ ] **Step 2: Connect Formatter to `DataGrid.vue`**

Update `DataGrid.vue` to accept `format: string` and `dataType: string` as props, formatting displayed values dynamically.

---

### Task 3: Communication Traffic Log Drawer (`TrafficLog.vue`)

**Files:**
- Create: `src/components/TrafficLog.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Displays real-time TX/RX frame history with timestamp, direction tag (TX in blue, RX in green), raw hex string, and auto-scroll control.

- [ ] **Step 1: Create `src/components/TrafficLog.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { ChevronUp, ChevronDown, Trash2, ArrowUpRight, ArrowDownLeft } from 'lucide-vue-next';

export interface LogEntry {
  id: string;
  time: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
}

const props = defineProps<{
  logs: LogEntry[];
}>();

const emit = defineEmits<{
  (e: 'clear'): void;
}>();

const isOpen = ref<boolean>(false);
</script>

<template>
  <div class="border-t border-gray-200 bg-white select-none transition-all">
    <!-- Header bar -->
    <div
      @click="isOpen = !isOpen"
      class="flex items-center justify-between px-4 py-2 bg-gray-50/80 hover:bg-gray-100/80 cursor-pointer border-b border-gray-200/50"
    >
      <div class="flex items-center gap-2 text-xs font-semibold text-gray-700">
        <span>Traffic Log</span>
        <span class="px-2 py-0.5 rounded-full bg-gray-200 text-gray-700 font-mono text-[10px]">{{ logs.length }} entries</span>
      </div>

      <div class="flex items-center gap-3" @click.stop>
        <button @click="emit('clear')" class="p-1 text-gray-400 hover:text-red-600 rounded transition" title="Clear log">
          <Trash2 class="w-3.5 h-3.5" />
        </button>
        <button @click="isOpen = !isOpen" class="p-1 text-gray-500 hover:text-gray-700 rounded transition">
          <component :is="isOpen ? ChevronDown : ChevronUp" class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Content Panel -->
    <div v-if="isOpen" class="h-40 overflow-auto p-3 font-mono text-xs bg-gray-900 text-gray-200 space-y-1.5">
      <div v-if="logs.length === 0" class="text-gray-500 text-center py-4 italic">No traffic recorded yet. Connect to start polling.</div>
      <div v-for="log in logs" :key="log.id" class="flex items-start gap-3 py-0.5 border-b border-gray-800/50 last:border-b-0">
        <span class="text-gray-500 select-none">{{ log.time }}</span>
        <span
          :class="[
            'px-1.5 py-0.5 rounded text-[10px] font-bold flex items-center gap-1 shrink-0',
            log.direction === 'TX' ? 'bg-blue-900/60 text-blue-300 border border-blue-700/50' : 'bg-emerald-900/60 text-emerald-300 border border-emerald-700/50'
          ]"
        >
          <component :is="log.direction === 'TX' ? ArrowUpRight : ArrowDownLeft" class="w-3 h-3" />
          {{ log.direction }}
        </span>
        <span class="text-gray-300 font-semibold shrink-0">{{ log.message }}</span>
        <span class="text-amber-300/90 break-all select-all font-mono">{{ log.bytes }}</span>
      </div>
    </div>
  </div>
</template>
```

---

### Task 4: Real-time Simulation Polling Ticker & Data Export (`src/utils/export.ts` & `App.vue`)

**Files:**
- Create: `src/utils/export.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: Create `src/utils/export.ts`**

```typescript
export function exportRegistersToJSON(sessionName: string, config: any, values: Record<number, number>) {
  const data = {
    session: sessionName,
    exportedAt: new Date().toISOString(),
    config,
    registers: values,
  };
  const jsonStr = JSON.stringify(data, null, 2);
  const blob = new Blob([jsonStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${sessionName.replace(/\s+/g, '_')}_registers.json`;
  a.click();
  URL.revokeObjectURL(url);
}
```

- [ ] **Step 2: Integration & Live Ticker in `App.vue`**

Add `setInterval` polling simulator when `connected === true`. Every `interval` ms:
- Generate a TX frame log e.g. `[TX] 01 03 00 00 00 28 C5 D3`
- Generate a RX frame log e.g. `[RX] 01 03 50 00 05 00 12 ...`
- Update simulated register values dynamically if raw toggle or interval is active.

---

### Task 5: Full Assembly & Build Verification

- Verify all components (`SettingsModal.vue`, `TrafficLog.vue`, `DataGrid.vue`, `Toolbar.vue`, `ConfigRow.vue`, `TabBar.vue`, `App.vue`) work seamlessly together.
- Run `pnpm build` (`vue-tsc --noEmit && vite build`) to ensure zero errors.
