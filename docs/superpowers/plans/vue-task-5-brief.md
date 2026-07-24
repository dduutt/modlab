# Task 5 Brief: Status Bar & App Integration for Vue 3

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Components created so far: `TabBar.vue`, `Toolbar.vue`, `ConfigRow.vue`, `DataGrid.vue`

## Instructions
1. Create `src/components/StatusBar.vue` for status indicator dot, status message, and copyright string.
2. Update `src/App.vue` to integrate `TabBar`, `Toolbar`, `ConfigRow`, `DataGrid`, and `StatusBar` into a cohesive, responsive 1000x700 window application UI.

Code for `src/components/StatusBar.vue`:
```vue
<script setup lang="ts">
defineProps<{
  message: string;
  type: 'error' | 'success' | 'info';
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-2 bg-white border-t border-gray-200 text-xs select-none">
    <div class="flex items-center gap-2">
      <span
        :class="[
          'w-2 h-2 rounded-full',
          type === 'error' ? 'bg-red-500 animate-pulse' : type === 'success' ? 'bg-emerald-500' : 'bg-blue-500'
        ]"
      ></span>
      <span
        :class="[
          'font-medium',
          type === 'error' ? 'text-red-600' : type === 'success' ? 'text-emerald-700' : 'text-gray-600'
        ]"
      >
        {{ message }}
      </span>
    </div>
    <div class="text-gray-400 font-mono">
      © Modlab • dote27@163.com
    </div>
  </div>
</template>
```

Code for `src/App.vue`:
```vue
<script setup lang="ts">
import { ref, reactive } from 'vue';
import TabBar from './components/TabBar.vue';
import Toolbar from './components/Toolbar.vue';
import ConfigRow, { type ModbusConfig } from './components/ConfigRow.vue';
import DataGrid from './components/DataGrid.vue';
import StatusBar from './components/StatusBar.vue';

interface SessionTab {
  id: string;
  title: string;
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
  config: ModbusConfig;
  values: Record<number, number>;
  statusMessage: string;
  statusType: 'error' | 'success' | 'info';
}

const tabs = ref<SessionTab[]>([
  {
    id: 'slave-1',
    title: 'Slave 2',
    role: 'Slave',
    protocol: 'TCP',
    ip: '0.0.0.0',
    port: 502,
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
    statusMessage: '[Slave 2] Connect before writing.',
    statusType: 'error',
  },
]);

const activeTabId = ref<string>('slave-1');

const activeTab = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0];
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
    role: 'Slave',
    protocol: 'TCP',
    ip: '0.0.0.0',
    port: 502 + count,
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
    activeTab.value.statusMessage = `[${activeTab.value.title}] Connected to ${activeTab.value.ip}:${activeTab.value.port}`;
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
      :role="activeTab.role"
      :protocol="activeTab.protocol"
      :ip="activeTab.ip"
      :port="activeTab.port"
      :connected="activeTab.connected"
      @toggle-connect="handleToggleConnect"
      @open-settings="() => {}"
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
      @update-cell="handleUpdateCell"
    />

    <StatusBar
      v-if="activeTab"
      :message="activeTab.statusMessage"
      :type="activeTab.statusType"
    />
  </div>
</template>
```

Write full report to `file:///F:/pro/modlab/docs/superpowers/plans/vue-task-5-report.md`.
