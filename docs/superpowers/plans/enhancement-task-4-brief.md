# Enhancement Task 4 Brief: Export Utilities & Full App Integration

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)

## Instructions
1. Create `src/utils/export.ts` to export register configuration & mapping to a JSON file.
2. Update `src/App.vue` to integrate:
   - `SettingsModal.vue` (opens on Settings button click, updates connection parameters)
   - `TrafficLog.vue` (bottom drawer displaying live TX/RX Hex frames)
   - Live Ticker Simulator (`setInterval` when `connected === true` to produce periodic Modbus polling frames & register updates)
   - Real-time format binding for `DataGrid.vue`

Code for `src/utils/export.ts`:
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

Code for `src/App.vue`:
```vue
<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue';
import TabBar from './components/TabBar.vue';
import Toolbar from './components/Toolbar.vue';
import ConfigRow, { type ModbusConfig } from './components/ConfigRow.vue';
import DataGrid from './components/DataGrid.vue';
import StatusBar from './components/StatusBar.vue';
import SettingsModal, { type ConnectionConfig } from './components/SettingsModal.vue';
import TrafficLog, { type LogEntry } from './components/TrafficLog.vue';

interface SessionTab {
  id: string;
  title: string;
  connection: ConnectionConfig;
  connected: boolean;
  config: ModbusConfig;
  values: Record<number, number>;
  logs: LogEntry[];
  statusMessage: string;
  statusType: 'error' | 'success' | 'info';
}

const tabs = ref<SessionTab[]>([
  {
    id: 'slave-1',
    title: 'Slave 2',
    connection: {
      role: 'Slave',
      protocol: 'TCP',
      ip: '0.0.0.0',
      port: 502,
      serialPort: 'COM1',
      baudRate: 9600,
      dataBits: 8,
      stopBits: 1,
      parity: 'None',
    },
    connected: false,
    config: {
      unitId: 1,
      functionCode: '0x03',
      startAddress: 0,
      count: 40,
      dataType: 'Int16',
      format: 'Dec',
      byteOrder: 'ABCD',
      interval: 1000,
      raw: false,
    },
    values: {},
    logs: [],
    statusMessage: '[Slave 2] Connect before writing.',
    statusType: 'error',
  },
]);

const activeTabId = ref<string>('slave-1');
const showSettings = ref<boolean>(false);

const activeTab = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0];
});

// Simulation Polling Timer
let timer: ReturnType<typeof setInterval> | null = null;

watch(() => activeTab.value?.connected, (isConnected) => {
  if (timer) clearInterval(timer);
  if (isConnected && activeTab.value) {
    timer = setInterval(() => {
      if (!activeTab.value || !activeTab.value.connected) return;
      const time = new Date().toLocaleTimeString();
      const unit = activeTab.value.config.unitId.toString(16).padStart(2, '0');
      const start = activeTab.value.config.startAddress.toString(16).padStart(4, '0');
      const count = activeTab.value.config.count.toString(16).padStart(4, '0');

      // Add TX Log
      activeTab.value.logs.unshift({
        id: Math.random().toString(36).slice(2),
        time,
        direction: 'TX',
        message: `Read Holding (0x03) Req`,
        bytes: `${unit} 03 ${start.slice(0, 2)} ${start.slice(2)} ${count.slice(0, 2)} ${count.slice(2)} C5 D3`,
      });

      // Add RX Log
      activeTab.value.logs.unshift({
        id: Math.random().toString(36).slice(2),
        time,
        direction: 'RX',
        message: `Read Holding (0x03) Resp`,
        bytes: `${unit} 03 50 ${Array.from({ length: 8 }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, '0')).join(' ')} ...`,
      });

      // Keep logs array capped at 100 entries
      if (activeTab.value.logs.length > 100) {
        activeTab.value.logs.pop();
      }

      // If Raw mode active, auto update values
      if (activeTab.value.config.raw) {
        const addr = activeTab.value.config.startAddress + Math.floor(Math.random() * activeTab.value.config.count);
        activeTab.value.values[addr] = Math.floor(Math.random() * 65535);
      }
    }, activeTab.value.config.interval || 1000);
  }
}, { immediate: true });

onUnmounted(() => {
  if (timer) clearInterval(timer);
});

function handleSelectTab(id: string) {
  activeTabId.value = id;
}

function handleAddTab() {
  const newId = `slave-${Date.now()}`;
  const count = tabs.value.length + 1;
  tabs.value.push({
    id: newId,
    title: `Slave ${count}`,
    connection: {
      role: 'Slave',
      protocol: 'TCP',
      ip: '0.0.0.0',
      port: 502 + count - 1,
      serialPort: `COM${count}`,
      baudRate: 9600,
      dataBits: 8,
      stopBits: 1,
      parity: 'None',
    },
    connected: false,
    config: {
      unitId: count,
      functionCode: '0x03',
      startAddress: 0,
      count: 40,
      dataType: 'Int16',
      format: 'Dec',
      byteOrder: 'ABCD',
      interval: 1000,
      raw: false,
    },
    values: {},
    logs: [],
    statusMessage: `[Slave ${count}] Idle.`,
    statusType: 'info',
  });
  activeTabId.value = newId;
}

function handleCloseTab(id: string) {
  if (tabs.value.length <= 1) return;
  tabs.value = tabs.value.filter(t => t.id !== id);
  if (activeTabId.value === id) {
    activeTabId.value = tabs.value[0].id;
  }
}

function handleToggleConnect() {
  if (!activeTab.value) return;
  activeTab.value.connected = !activeTab.value.connected;
  if (activeTab.value.connected) {
    const connStr = activeTab.value.connection.protocol === 'TCP'
      ? `${activeTab.value.connection.ip}:${activeTab.value.connection.port}`
      : `${activeTab.value.connection.serialPort} (${activeTab.value.connection.baudRate})`;
    activeTab.value.statusMessage = `[${activeTab.value.title}] Connected to ${connStr}`;
    activeTab.value.statusType = 'success';
  } else {
    activeTab.value.statusMessage = `[${activeTab.value.title}] Disconnected.`;
    activeTab.value.statusType = 'info';
  }
}

function handleFillRandom() {
  if (!activeTab.value) return;
  const newValues = { ...activeTab.value.values };
  for (let addr = activeTab.value.config.startAddress; addr < activeTab.value.config.startAddress + activeTab.value.config.count; addr++) {
    newValues[addr] = Math.floor(Math.random() * 1000);
  }
  activeTab.value.values = newValues;
}

function handleFillIncrement() {
  if (!activeTab.value) return;
  const newValues = { ...activeTab.value.values };
  let val = 1;
  for (let addr = activeTab.value.config.startAddress; addr < activeTab.value.config.startAddress + activeTab.value.config.count; addr++) {
    newValues[addr] = val++;
  }
  activeTab.value.values = newValues;
}

function handleUpdateCell(address: number, value: number) {
  if (!activeTab.value) return;
  activeTab.value.values[address] = value;
}

function handleSaveSettings(newConnection: ConnectionConfig) {
  if (!activeTab.value) return;
  activeTab.value.connection = newConnection;
  activeTab.value.title = `${newConnection.role} ${tabs.value.indexOf(activeTab.value) + 1}`;
}
</script>

<template>
  <div class="flex flex-col h-screen w-screen bg-gray-100 overflow-hidden text-gray-800 font-sans">
    <TabBar
      :tabs="tabs"
      :activeTabId="activeTabId"
      @select-tab="handleSelectTab"
      @add-tab="handleAddTab"
      @close-tab="handleCloseTab"
    />

    <Toolbar
      v-if="activeTab"
      :role="activeTab.connection.role"
      :protocol="activeTab.connection.protocol"
      :ip="activeTab.connection.ip"
      :port="activeTab.connection.port"
      :connected="activeTab.connected"
      @toggle-connect="handleToggleConnect"
      @open-settings="showSettings = true"
      @fill-random="handleFillRandom"
      @fill-increment="handleFillIncrement"
    />

    <ConfigRow
      v-if="activeTab"
      v-model:config="activeTab.config"
    />

    <DataGrid
      v-if="activeTab"
      :startAddress="activeTab.config.startAddress"
      :count="activeTab.config.count"
      :values="activeTab.values"
      :format="activeTab.config.format"
      :dataType="activeTab.config.dataType"
      @update-cell="handleUpdateCell"
    />

    <TrafficLog
      v-if="activeTab"
      :logs="activeTab.logs"
      @clear="activeTab.logs = []"
    />

    <StatusBar
      v-if="activeTab"
      :message="activeTab.statusMessage"
      :type="activeTab.statusType"
    />

    <SettingsModal
      v-if="activeTab"
      :show="showSettings"
      :config="activeTab.connection"
      @close="showSettings = false"
      @save="handleSaveSettings"
    />
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/enhancement-task-4-report.md`.
