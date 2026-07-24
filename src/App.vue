<script setup lang="ts">
import { ref, computed, watch, onUnmounted, onMounted } from 'vue';
import TabBar from './components/TabBar.vue';
import Toolbar from './components/Toolbar.vue';
import ConfigRow, { type ModbusConfig } from './components/ConfigRow.vue';
import DataGrid from './components/DataGrid.vue';
import StatusBar from './components/StatusBar.vue';
import SettingsModal, { type ConnectionConfig } from './components/SettingsModal.vue';
import TrafficLog, { type LogEntry } from './components/TrafficLog.vue';
import NewDeviceModal from './components/NewDeviceModal.vue';
import ConfirmModal from './components/ConfirmModal.vue';
import TrafficLogWindow from './components/TrafficLogWindow.vue';
import { ModbusService } from './services/modbusService';
import { generateRandomRegisters, incrementFormattedValue } from './utils/modbusFormatter';
import { emit as tauriEmit } from '@tauri-apps/api/event';

const isTrafficLogRoute = ref<boolean>(false);

onMounted(() => {
  if (window.location.hash === '#/traffic-log') {
    isTrafficLogRoute.value = true;
  }
  window.addEventListener('hashchange', () => {
    isTrafficLogRoute.value = window.location.hash === '#/traffic-log';
  });
});

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
  autoIncrement?: boolean;
  isPolling?: boolean;
}

const STORAGE_KEY = 'modlab_session_configs_v1';

function getDefaultTabs(): SessionTab[] {
  return [
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
        timeoutMs: 1000,
        retries: 3,
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
  ];
}

function loadSavedState(): { loadedTabs: SessionTab[]; savedActiveId: string } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && Array.isArray(parsed.tabs) && parsed.tabs.length > 0) {
        const loadedTabs: SessionTab[] = parsed.tabs.map((t: any) => ({
          id: t.id || `tab-${Math.random().toString(36).slice(2)}`,
          title: t.title || 'Device',
          connection: t.connection || {
            role: 'Slave',
            protocol: 'TCP',
            ip: '0.0.0.0',
            port: 502,
            serialPort: 'COM1',
            baudRate: 9600,
            dataBits: 8,
            stopBits: 1,
            parity: 'None',
            timeoutMs: 1000,
            retries: 3,
          },
          connected: false, // ALWAYS unconnected on startup!
          config: t.config || {
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
          values: {}, // ALWAYS empty values on startup!
          logs: [],   // ALWAYS empty logs on startup!
          statusMessage: `[${t.title || 'Device'}] Disconnected.`,
          statusType: 'info',
        }));
        const savedActiveId = loadedTabs.some(t => t.id === parsed.activeTabId)
          ? parsed.activeTabId
          : loadedTabs[0].id;
        return { loadedTabs, savedActiveId };
      }
    }
  } catch (e) {
    console.error('Failed to load session configs:', e);
  }
  return { loadedTabs: getDefaultTabs(), savedActiveId: 'slave-1' };
}

const { loadedTabs, savedActiveId } = loadSavedState();
const tabs = ref<SessionTab[]>(loadedTabs);
const activeTabId = ref<string>(savedActiveId);

watch(
  [tabs, activeTabId],
  () => {
    try {
      const payload = {
        activeTabId: activeTabId.value,
        tabs: tabs.value.map(t => ({
          id: t.id,
          title: t.title,
          connection: t.connection,
          config: t.config,
        })),
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
    } catch (e) {
      console.error('Failed to save session configs:', e);
    }
  },
  { deep: true }
);
const showSettings = ref<boolean>(false);
const showNewDeviceModal = ref<boolean>(false);
const showConfirmModal = ref<boolean>(false);
const showLogPanel = ref<boolean>(false);
const tabToCloseId = ref<string | null>(null);

const activeTab = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0];
});

const existingNames = computed(() => {
  return tabs.value.map(t => t.title);
});

const allLogs = computed(() => {
  const aggregated: LogEntry[] = [];
  for (const tab of tabs.value) {
    aggregated.push(...tab.logs);
  }
  return aggregated;
});

// Broadcast log entry via Tauri Event Bus
function broadcastLog(session: SessionTab, direction: 'TX' | 'RX', message: string, bytes: string) {
  const timestamp = new Date().toLocaleTimeString();
  const logEntry = {
    id: Math.random().toString(36).slice(2),
    sessionId: session.id,
    sessionTitle: session.title,
    direction,
    message,
    bytes,
    timestamp,
  };

  session.logs.unshift({
    id: logEntry.id,
    time: timestamp,
    deviceName: session.title,
    direction,
    message,
    bytes,
  });
  if (session.logs.length > 100) session.logs.pop();

  try {
    tauriEmit('traffic-log-entry', logEntry).catch(() => {});
  } catch (e) {
    // Ignore in non-Tauri
  }
}

// Robust Multi-Tab Timer Management
const tabTimers = new Map<string, { pollTimer?: ReturnType<typeof setInterval>; autoIncTimer?: ReturnType<typeof setInterval> }>();

function clearTabTimers(tabId: string) {
  const existing = tabTimers.get(tabId);
  if (existing) {
    if (existing.pollTimer) clearInterval(existing.pollTimer);
    if (existing.autoIncTimer) clearInterval(existing.autoIncTimer);
    tabTimers.delete(tabId);
  }
}

function updateTabTimers(tab: SessionTab) {
  let timers = tabTimers.get(tab.id) || {};
  if (timers.pollTimer) {
    clearInterval(timers.pollTimer);
    timers.pollTimer = undefined;
  }
  if (timers.autoIncTimer) {
    clearInterval(timers.autoIncTimer);
    timers.autoIncTimer = undefined;
  }

  // 1. Master Polling Timer
  if (tab.connection.role === 'Master' && tab.connected && tab.isPolling) {
    timers.pollTimer = setInterval(async () => {
      if (!tab.connected || !tab.isPolling) {
        if (timers.pollTimer) clearInterval(timers.pollTimer);
        timers.pollTimer = undefined;
        return;
      }
      const unit = tab.config.unitId.toString(16).padStart(2, '0');
      const start = tab.config.startAddress.toString(16).padStart(4, '0');
      const count = tab.config.count.toString(16).padStart(4, '0');

      const txBytes = `${unit} 03 ${start.slice(0, 2)} ${start.slice(2)} ${count.slice(0, 2)} ${count.slice(2)} C5 D3`;
      broadcastLog(tab, 'TX', `Read (${tab.config.functionCode}) Req`, txBytes);

      try {
        const fetched = await ModbusService.readRegisters(tab.id, tab.config.startAddress, tab.config.count);
        if (fetched && Object.keys(fetched).length > 0) {
          tab.values = { ...tab.values, ...fetched };
        }
      } catch (err) {
        console.error(`[${tab.title}] Poll error:`, err);
      }

      const rxBytes = `${unit} 03 50 ${Array.from({ length: 8 }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, '0')).join(' ')} ...`;
      broadcastLog(tab, 'RX', `Read (${tab.config.functionCode}) Resp`, rxBytes);
    }, tab.config.interval || 1000);
  }

  // 2. Slave Auto Increment Timer
  if (tab.connection.role === 'Slave' && tab.connected && tab.autoIncrement) {
    timers.autoIncTimer = setInterval(async () => {
      if (!tab.connected || !tab.autoIncrement) {
        if (timers.autoIncTimer) clearInterval(timers.autoIncTimer);
        timers.autoIncTimer = undefined;
        return;
      }
      const cfg = tab.config;
      const is32 = cfg.dataType === 'Float32' || cfg.dataType === 'Int32' || cfg.dataType === 'UInt32';
      const step = is32 ? 2 : 1;
      const newValues = { ...tab.values };

      for (let addr = cfg.startAddress; addr < cfg.startAddress + cfg.count; addr += step) {
        const raw1 = newValues[addr] ?? 0;
        const raw2 = is32 ? (newValues[addr + 1] ?? 0) : undefined;
        const inc = incrementFormattedValue(raw1, cfg.format, cfg.dataType, cfg.byteOrder, raw2);
        newValues[addr] = inc.word1;
        await ModbusService.writeRegister(tab.id, addr, inc.word1).catch(() => {});
        if (is32 && inc.word2 !== undefined && addr + 1 < cfg.startAddress + cfg.count) {
          newValues[addr + 1] = inc.word2;
          await ModbusService.writeRegister(tab.id, addr + 1, inc.word2).catch(() => {});
        }
      }
      tab.values = newValues;
    }, tab.config.interval || 1000);
  }

  tabTimers.set(tab.id, timers);
}

// Watch tabs state deeply and manage timers for every tab independently
watch(
  () => tabs.value.map(t => ({
    id: t.id,
    connected: t.connected,
    isPolling: t.isPolling,
    autoIncrement: t.autoIncrement,
    role: t.connection.role,
    interval: t.config.interval,
    startAddress: t.config.startAddress,
    count: t.config.count,
    dataType: t.config.dataType,
    format: t.config.format,
    byteOrder: t.config.byteOrder,
  })),
  (tabStates) => {
    for (const tabState of tabStates) {
      const realTab = tabs.value.find(t => t.id === tabState.id);
      if (realTab) {
        // Disconnection automatically shuts down polling and autoIncrement timers
        if (!realTab.connected) {
          if (realTab.isPolling) realTab.isPolling = false;
          if (realTab.autoIncrement) realTab.autoIncrement = false;
        }
        updateTabTimers(realTab);
      }
    }
  },
  { deep: true, immediate: true }
);

onUnmounted(() => {
  for (const tabId of tabTimers.keys()) {
    clearTabTimers(tabId);
  }
});

function handleSelectTab(id: string) {
  activeTabId.value = id;
}

function handleRenameTab(id: string, newTitle: string) {
  const tab = tabs.value.find(t => t.id === id);
  if (tab) {
    tab.title = newTitle;
  }
}

function handleCreateDevice(name: string, role: 'Slave' | 'Master') {
  const newId = `${role.toLowerCase()}-${Date.now()}`;
  const count = tabs.value.length + 1;
  tabs.value.push({
    id: newId,
    title: name,
    connection: {
      role,
      protocol: 'TCP',
      ip: role === 'Master' ? '127.0.0.1' : '0.0.0.0',
      port: 502 + count - 1,
      serialPort: `COM${count}`,
      baudRate: 9600,
      dataBits: 8,
      stopBits: 1,
      parity: 'None',
      timeoutMs: 1000,
      retries: 3,
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
    statusMessage: `[${name}] Idle.`,
    statusType: 'info',
  });
  activeTabId.value = newId;
}

function handleCloseTabRequest(id: string) {
  if (tabs.value.length <= 1) return;
  const tab = tabs.value.find(t => t.id === id);
  if (!tab) return;
  if (tab.connected) {
    tabToCloseId.value = id;
    showConfirmModal.value = true;
  } else {
    handleCloseTab(id);
  }
}

function handleConfirmCloseTab() {
  if (tabToCloseId.value) {
    const tab = tabs.value.find(t => t.id === tabToCloseId.value);
    if (tab && tab.connected) {
      ModbusService.disconnect(tab.id).catch(() => {});
      tab.connected = false;
    }
    handleCloseTab(tabToCloseId.value);
  }
  showConfirmModal.value = false;
  tabToCloseId.value = null;
}

function handleCloseTab(id: string) {
  if (tabs.value.length <= 1) return;
  clearTabTimers(id);
  tabs.value = tabs.value.filter(t => t.id !== id);
  if (activeTabId.value === id) {
    activeTabId.value = tabs.value[0].id;
  }
}

async function handleToggleConnect() {
  if (!activeTab.value) return;
  if (activeTab.value.connected) {
    try {
      broadcastLog(activeTab.value, 'TX', `Disconnect Req`, `Session: ${activeTab.value.title}`);
      await ModbusService.disconnect(activeTab.value.id);
      activeTab.value.connected = false;
      activeTab.value.isPolling = false;
      activeTab.value.autoIncrement = false;
      clearTabTimers(activeTab.value.id);
      activeTab.value.statusMessage = `[${activeTab.value.title}] Disconnected.`;
      activeTab.value.statusType = 'info';
      broadcastLog(activeTab.value, 'RX', `Disconnected OK`, `Status: Closed`);
    } catch (err: any) {
      activeTab.value.statusMessage = `[${activeTab.value.title}] Disconnect failed: ${err?.message || err}`;
      activeTab.value.statusType = 'error';
    }
  } else {
    try {
      broadcastLog(activeTab.value, 'TX', `${activeTab.value.connection.role} Connect (${activeTab.value.connection.protocol})`, `${activeTab.value.connection.ip}:${activeTab.value.connection.port}`);
      const msg = await ModbusService.connect(activeTab.value.id, activeTab.value.connection);
      activeTab.value.connected = true;
      activeTab.value.statusMessage = `[${activeTab.value.title}] ${msg}`;
      activeTab.value.statusType = 'success';
      broadcastLog(activeTab.value, 'RX', `${activeTab.value.connection.role} Connected OK`, `Status: Active`);
    } catch (err: any) {
      activeTab.value.statusMessage = `[${activeTab.value.title}] Connection failed: ${err?.message || err}`;
      activeTab.value.statusType = 'error';
    }
  }
}

async function handleFillRandom() {
  if (!activeTab.value) return;
  try {
    broadcastLog(activeTab.value, 'TX', `Random Fill Req`, `Start: ${activeTab.value.config.startAddress}, Count: ${activeTab.value.config.count}`);
    const updated = generateRandomRegisters(
      activeTab.value.config.startAddress,
      activeTab.value.config.count,
      activeTab.value.config.dataType,
      activeTab.value.config.byteOrder,
      activeTab.value.config.functionCode
    );
    activeTab.value.values = { ...activeTab.value.values, ...updated };
    for (const [addrStr, val] of Object.entries(updated)) {
      await ModbusService.writeRegister(activeTab.value.id, Number(addrStr), val).catch(() => {});
    }
    broadcastLog(activeTab.value, 'RX', `Random Fill Resp OK`, `Updated ${activeTab.value.config.count} registers`);
  } catch (err) {
    console.error('Failed fill random:', err);
  }
}

function handleToggleAutoIncrement() {
  if (!activeTab.value) return;
  activeTab.value.autoIncrement = !activeTab.value.autoIncrement;
}

function handleTogglePoll() {
  if (!activeTab.value) return;
  activeTab.value.isPolling = !activeTab.value.isPolling;
}

async function handleReadOnce() {
  if (!activeTab.value || !activeTab.value.connected) return;
  const unit = activeTab.value.config.unitId.toString(16).padStart(2, '0');
  const start = activeTab.value.config.startAddress.toString(16).padStart(4, '0');
  const count = activeTab.value.config.count.toString(16).padStart(4, '0');

  const txBytes = `${unit} 03 ${start.slice(0, 2)} ${start.slice(2)} ${count.slice(0, 2)} ${count.slice(2)} C5 D3`;
  broadcastLog(activeTab.value, 'TX', `Read (${activeTab.value.config.functionCode}) Req`, txBytes);

  try {
    const fetched = await ModbusService.readRegisters(
      activeTab.value.id,
      activeTab.value.config.startAddress,
      activeTab.value.config.count
    );
    if (Object.keys(fetched).length > 0) {
      activeTab.value.values = { ...activeTab.value.values, ...fetched };
    }
  } catch (err) {
    console.error('Read failed:', err);
  }

  const rxBytes = `${unit} 03 50 ${Array.from({ length: 8 }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, '0')).join(' ')} ...`;
  broadcastLog(activeTab.value, 'RX', `Read (${activeTab.value.config.functionCode}) Resp`, rxBytes);
}

async function handleUpdateCell(address: number, value: number) {
  if (!activeTab.value) return;
  activeTab.value.values[address] = value;
  broadcastLog(activeTab.value, 'TX', `Write Register (Addr: ${address})`, `Val: ${value}`);
  try {
    await ModbusService.writeRegister(activeTab.value.id, address, value);
    broadcastLog(activeTab.value, 'RX', `Write Register Resp OK`, `Addr: ${address}`);
  } catch (err: any) {
    console.error('Failed to write register:', err);
  }
}

async function handleUpdateCellPair(address1: number, value1: number, address2: number, value2: number) {
  if (!activeTab.value) return;
  activeTab.value.values[address1] = value1;
  activeTab.value.values[address2] = value2;
  broadcastLog(activeTab.value, 'TX', `Write Register Pair (Addr: ${address1}-${address2})`, `Word1: ${value1}, Word2: ${value2}`);
  try {
    await ModbusService.writeRegister(activeTab.value.id, address1, value1);
    await ModbusService.writeRegister(activeTab.value.id, address2, value2);
    broadcastLog(activeTab.value, 'RX', `Write Register Pair Resp OK`, `Addr: ${address1}-${address2}`);
  } catch (err: any) {
    console.error('Failed to write register pair:', err);
  }
}

function handleSaveSettings(newConnection: ConnectionConfig) {
  if (!activeTab.value) return;
  activeTab.value.connection = newConnection;
}


</script>

<template>
  <div v-if="isTrafficLogRoute" class="h-screen w-screen">
    <TrafficLogWindow />
  </div>

  <div v-else class="h-screen w-screen flex flex-col bg-gray-100 font-sans select-none overflow-hidden">
    <TabBar
      :tabs="tabs"
      :activeTabId="activeTabId"
      @select-tab="handleSelectTab"
      @add-tab="showNewDeviceModal = true"
      @close-tab="handleCloseTabRequest"
      @rename-tab="handleRenameTab"
    />

    <Toolbar
      v-if="activeTab"
      :role="activeTab.connection.role"
      :protocol="activeTab.connection.protocol"
      :ip="activeTab.connection.ip"
      :port="activeTab.connection.port"
      :connected="activeTab.connected"
      :functionCode="activeTab.config.functionCode"
      :autoIncrement="activeTab.autoIncrement"
      :isPolling="activeTab.isPolling"
      @toggle-connect="handleToggleConnect"
      @open-settings="showSettings = true"
      @toggle-auto-increment="handleToggleAutoIncrement"
      @fill-random="handleFillRandom"
      @toggle-poll="handleTogglePoll"
      @read-once="handleReadOnce"
    />

    <ConfigRow
      v-if="activeTab"
      v-model:config="activeTab.config"
      :connected="activeTab.connected"
      :role="activeTab.connection.role"
    />

    <DataGrid
      v-if="activeTab"
      :startAddress="activeTab.config.startAddress"
      :count="activeTab.config.count"
      :values="activeTab.values"
      :format="activeTab.config.format"
      :dataType="activeTab.config.dataType"
      :byteOrder="activeTab.config.byteOrder"
      :functionCode="activeTab.config.functionCode"
      :raw="activeTab.config.raw"
      @update-cell="handleUpdateCell"
      @update-cell-pair="handleUpdateCellPair"
    />

    <TrafficLog
      v-if="activeTab && showLogPanel"
      :logs="allLogs"
      :deviceNames="existingNames"
      @clear="activeTab ? (activeTab.logs = []) : null"
      @close="showLogPanel = false"
    />

    <StatusBar
      v-if="activeTab"
      :message="activeTab.statusMessage"
      :type="activeTab.statusType"
      :logCount="allLogs.length"
      :showLogPanel="showLogPanel"
      @toggle-logs="showLogPanel = !showLogPanel"
    />

    <SettingsModal
      v-if="activeTab"
      :show="showSettings"
      :config="activeTab.connection"
      @close="showSettings = false"
      @save="handleSaveSettings"
    />

    <NewDeviceModal
      :show="showNewDeviceModal"
      :existingNames="existingNames"
      @close="showNewDeviceModal = false"
      @create="handleCreateDevice"
    />

    <ConfirmModal
      :show="showConfirmModal"
      title="Close Active Connection"
      message="This session is currently connected to a device. Are you sure you want to disconnect and close it?"
      @close="showConfirmModal = false"
      @confirm="handleConfirmCloseTab"
    />
  </div>
</template>