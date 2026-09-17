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
import { appendLog } from './utils/logging';
import { ModbusService } from './services/modbusService';
import { generateRandomRegisters, incrementFormattedValue } from './utils/modbusFormatter';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { useI18n } from 'vue-i18n';

const { locale } = useI18n();
const language = computed<'en' | 'zh'>(() => locale.value as 'en' | 'zh');
function toggleLanguage() {
  locale.value = locale.value === 'en' ? 'zh' : 'en';
}

let unlistenTraffic: UnlistenFn | null = null;

onMounted(async () => {
  try {
    unlistenTraffic = await listen<any>('traffic-log-entry', (event) => {
      const entry = event.payload;
      if (!entry || !entry.sessionId || entry.origin === 'frontend') return;
      const targetTab = tabs.value.find(t => t.id === entry.sessionId);
      if (targetTab) {
        if (!targetTab.logs.some(l => l.id === entry.id)) {
          const log: LogEntry = {
            id: entry.id,
            time: entry.timestamp || new Date().toLocaleTimeString('en-GB', { hour12: false }) + '.' + String(Date.now() % 1000).padStart(3, '0'),
            deviceName: targetTab.title,
            sessionId: targetTab.id,
            timestampMs: entry.timestampMs ?? Date.now(),
            durationMs: entry.durationMs,
            level: entry.level || 'info',
            kind: entry.kind || 'frame',
            detail: entry.detail ?? true,
            direction: entry.direction,
            message: entry.message,
            bytes: entry.bytes,
            protocol: entry.protocol,
            complete: entry.complete,
            role: targetTab.connection.role,
          };
          log.time = new Date(log.timestampMs).toLocaleTimeString('en-GB', { hour12: false }) + '.' + String(log.timestampMs % 1000).padStart(3, '0');
          if (log.kind === 'frame' || log.level === 'error') targetTab.logs = appendLog(targetTab.logs, log);
          trimLogs(targetTab);
        }
      }
    });
  } catch (err) {
    console.error('Failed to register traffic-log-entry listener in App.vue:', err);
  }
});

interface SessionTab {
  id: string;
  title: string;
  connection: ConnectionConfig;
  connected: boolean;
  busy?: boolean;
  connectionVersion?: number;
  valueVersion?: number;
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
      statusMessage: '[Slave 2] Disconnected.',
      statusType: 'info',
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
    aggregated.push(...tab.logs.map(log => ({ ...log, deviceName: tab.title })));
  }
  return aggregated.sort((a, b) => b.timestampMs - a.timestampMs);
});

// Bound memory while retaining individual transfers in arrival order.
function trimLogs(session: SessionTab) {
  session.logs = session.logs.slice(0, 2000);
}

function broadcastLog(session: SessionTab, direction: 'TX' | 'RX', message: string, bytes: string, detail = false, startedAt?: number) {
  const timestampMs = Date.now();
  const level = /failed/i.test(message) ? 'error' : /restored/i.test(message) ? 'success' : 'info';
  const logEntry = {
    origin: 'frontend', id: crypto.randomUUID(), sessionId: session.id, sessionTitle: session.title,
    direction, message, bytes, timestampMs,
    durationMs: startedAt === undefined ? undefined : Math.max(0, timestampMs - startedAt),
    timestamp: new Date(timestampMs).toLocaleTimeString('en-GB', { hour12: false }) + '.' + String(timestampMs % 1000).padStart(3, '0'),
    kind: 'operation' as const, level: level as 'error' | 'success' | 'info',
    detail: (detail || direction === 'TX') && level !== 'error',
  };
  if (level !== 'error' || !/read|write|connect|disconnect/i.test(message)) return;
  session.logs = appendLog(session.logs, { ...logEntry, time: logEntry.timestamp, deviceName: session.title });
  trimLogs(session);
}

function logFailure(session: SessionTab, operation: string, error: unknown, startedAt?: number) {
  broadcastLog(session, 'RX', `${operation} Failed`, error instanceof Error ? error.message : String(error), false, startedAt);
}

function readTabRegisters(tab: SessionTab) {
  return ModbusService.readRegisters(
    tab.id,
    tab.config.startAddress,
    tab.config.count,
    tab.config.functionCode,
    tab.config.unitId,
    tab.connection.timeoutMs,
    tab.connection.retries,
  );
}

function writeTabRegister(tab: SessionTab, address: number, value: number) {
  return ModbusService.writeRegister(
    tab.id,
    address,
    value,
    tab.config.functionCode,
    tab.config.unitId,
    tab.connection.timeoutMs,
    tab.connection.retries,
  );
}

function captureTabRequest(tab: SessionTab) {
  const connection = tab.connection;
  const version = tab.connectionVersion;
  const { unitId, functionCode, startAddress, count } = tab.config;
  return () => tab.connected && !tab.busy && tab.connection === connection
    && tab.connectionVersion === version && tab.config.unitId === unitId
    && tab.config.functionCode === functionCode && tab.config.startAddress === startAddress
    && tab.config.count === count;
}

watch(() => tabs.value.map(tab => ({ id: tab.id, unitId: tab.config.unitId, functionCode: tab.config.functionCode })), (current, previous) => {
  for (const next of current) {
    const before = previous.find(tab => tab.id === next.id);
    if (before && (before.unitId !== next.unitId || before.functionCode !== next.functionCode)) {
      const tab = tabs.value.find(tab => tab.id === next.id)!;
      tab.values = {};
      tab.connectionVersion = (tab.connectionVersion ?? 0) + 1;
    }
  }
});

// Robust Multi-Tab Timer Management
const tabTimers = new Map<string, { pollTimer?: ReturnType<typeof setInterval>; autoIncTimer?: ReturnType<typeof setInterval> }>();
const pollingInFlight = new Set<string>();
const incrementInFlight = new Set<string>();

function clearTabTimers(tabId: string) {
  const existing = tabTimers.get(tabId);
  if (existing) {
    if (existing.pollTimer) clearInterval(existing.pollTimer);
    if (existing.autoIncTimer) clearInterval(existing.autoIncTimer);
    tabTimers.delete(tabId);
  }
}

function updateTabTimers(tab: SessionTab) {
  clearTabTimers(tab.id);
  const timers: { pollTimer?: ReturnType<typeof setInterval>; autoIncTimer?: ReturnType<typeof setInterval> } = {};

  // 1. Master Polling Timer
  if (tab.connection.role === 'Master' && tab.connected && tab.isPolling) {
    timers.pollTimer = setInterval(async () => {
      if (!tab.connected || !tab.isPolling) {
        if (timers.pollTimer) clearInterval(timers.pollTimer);
        timers.pollTimer = undefined;
        return;
      }
      if (pollingInFlight.has(tab.id) || tab.busy) return;
      pollingInFlight.add(tab.id);
      const operationStarted = Date.now();
      broadcastLog(tab, 'TX', `Read (${tab.config.functionCode}) Req`, `Unit: ${tab.config.unitId}, Start: ${tab.config.startAddress}, Count: ${tab.config.count}`, true);

      try {
        const fetched = await readTabRegisters(tab);
        if (tabTimers.get(tab.id) !== timers || !tab.connected) return;
        if (fetched && Object.keys(fetched).length > 0) {
          tab.values = { ...tab.values, ...fetched };
        }
        if (tab.statusType === 'error' && (tab.statusMessage.startsWith(`[${tab.title}] Poll failed:`) || tab.statusMessage.startsWith(`[${tab.title}] Read failed:`))) {
          broadcastLog(tab, 'RX', 'Communication restored', '');
          tab.statusMessage = `[${tab.title}] Communication restored.`;
          tab.statusType = 'success';
        }
        broadcastLog(tab, 'RX', `Read (${tab.config.functionCode}) Resp`, `Unit: ${tab.config.unitId}, Start: ${tab.config.startAddress}, Count: ${Object.keys(fetched).length}`, true, operationStarted);
      } catch (err) {
        if (tabTimers.get(tab.id) !== timers || !tab.connected) return;
        console.error(`[${tab.title}] Poll error:`, err);
        tab.statusMessage = `[${tab.title}] Poll failed: ${err instanceof Error ? err.message : err}`;
        tab.statusType = 'error';
        broadcastLog(tab, 'RX', `Read (${tab.config.functionCode}) Failed`, String(err), false, operationStarted);
      } finally {
        pollingInFlight.delete(tab.id);
      }
    }, tab.config.interval || 1000);
  }

  // 2. Slave Auto Increment Timer
  // Read the local slave memory so external master writes reach the grid.
  // This is a local snapshot, not evidence of successful wire communication.
  if (tab.connection.role === 'Slave' && tab.connected && !tab.autoIncrement) {
    timers.pollTimer = setInterval(async () => {
      if (!tab.connected || tab.busy || tab.autoIncrement || pollingInFlight.has(tab.id)) return;
      const isCurrent = captureTabRequest(tab);
      const valueVersion = tab.valueVersion;
      pollingInFlight.add(tab.id);
      try {
        const fetched = await readTabRegisters(tab);
        if (isCurrent() && tabTimers.get(tab.id) === timers && tab.valueVersion === valueVersion) {
          tab.values = { ...fetched };
          if (tab.statusMessage.startsWith(`[${tab.title}] Sync failed:`)) {
            broadcastLog(tab, 'RX', 'Sync restored', '');
            tab.statusMessage = `[${tab.title}] Listening.`;
            tab.statusType = 'info';
          }
        }
      } catch (err) {
        if (isCurrent() && tabTimers.get(tab.id) === timers) {
          if (!tab.statusMessage.startsWith(`[${tab.title}] Sync failed:`)) logFailure(tab, 'Sync', err);
          tab.statusMessage = `[${tab.title}] Sync failed: ${err instanceof Error ? err.message : err}`;
          tab.statusType = 'error';
        }
      } finally {
        pollingInFlight.delete(tab.id);
      }
    }, 1000);
  }

  if (tab.connection.role === 'Slave' && tab.connected && tab.autoIncrement) {
    timers.autoIncTimer = setInterval(async () => {
      if (!tab.connected || !tab.autoIncrement) {
        if (timers.autoIncTimer) clearInterval(timers.autoIncTimer);
        timers.autoIncTimer = undefined;
        return;
      }
      if (incrementInFlight.has(tab.id) || tab.busy) return;
      incrementInFlight.add(tab.id);
      try {
        const cfg = { ...tab.config };
        const isCoil = cfg.functionCode === '0x01' || cfg.functionCode === '0x02';
        const is32 = !isCoil && (cfg.dataType === 'Float32' || cfg.dataType === 'Int32' || cfg.dataType === 'UInt32');
        const step = is32 ? 2 : 1;
        const newValues = { ...await readTabRegisters(tab) };
        if (!tab.connected || tab.busy || tabTimers.get(tab.id) !== timers) return;
        let completed = true;

        for (let addr = cfg.startAddress; addr < cfg.startAddress + cfg.count; addr += step) {
          if (!tab.connected || tab.busy || !tab.autoIncrement || tabTimers.get(tab.id) !== timers) return;
          if (is32 && addr + 1 >= cfg.startAddress + cfg.count) break;
          const raw1 = newValues[addr] ?? 0;
          const raw2 = is32 ? (newValues[addr + 1] ?? 0) : undefined;
          try {
            const inc = incrementFormattedValue(raw1, cfg.format, isCoil ? 'Coil' : cfg.dataType, cfg.byteOrder, raw2);
            if (is32 && inc.word2 !== undefined && addr + 1 < cfg.startAddress + cfg.count) {
              await ModbusService.writeRegisters(
                tab.id,
                addr,
                [inc.word1, inc.word2],
                tab.config.functionCode,
                tab.config.unitId,
                tab.connection.timeoutMs,
                tab.connection.retries
              );
              newValues[addr] = inc.word1;
              newValues[addr + 1] = inc.word2;
            } else {
              await writeTabRegister(tab, addr, inc.word1);
              newValues[addr] = inc.word1;
            }
          } catch (err: any) {
          if (!tab.statusMessage.startsWith(`[${tab.title}] Auto increment failed:`)) logFailure(tab, 'Auto increment', err);
            tab.statusMessage = `[${tab.title}] Auto increment failed: ${err?.message || err}`;
            tab.statusType = 'error';
            completed = false;
            break;
          }
        }
        if (tab.connected && tabTimers.get(tab.id) === timers) {
          tab.values = newValues;
          if (completed && tab.statusType === 'error' && tab.statusMessage.startsWith(`[${tab.title}] Auto increment failed:`)) {
            broadcastLog(tab, 'RX', 'Auto increment restored', '');
            tab.statusMessage = `[${tab.title}] Listening.`;
            tab.statusType = 'info';
          }
        }
      } catch (err) {
        if (tab.connected && tabTimers.get(tab.id) === timers) {
          if (!tab.statusMessage.startsWith(`[${tab.title}] Auto increment failed:`)) logFailure(tab, 'Auto increment', err);
          tab.statusMessage = `[${tab.title}] Auto increment failed: ${err instanceof Error ? err.message : err}`;
          tab.statusType = 'error';
        }
      } finally {
        incrementInFlight.delete(tab.id);
      }
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
    functionCode: t.config.functionCode,
    unitId: t.config.unitId,
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
  if (unlistenTraffic) {
    unlistenTraffic();
  }
  for (const tabId of tabTimers.keys()) {
    clearTabTimers(tabId);
  }
});

function handleSelectTab(id: string) {
  showSettings.value = false;
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
  if (tab.busy) return;
  if (tab.connected) {
    tabToCloseId.value = id;
    showConfirmModal.value = true;
  } else {
    handleCloseTab(id);
  }
}

async function handleConfirmCloseTab() {
  const id = tabToCloseId.value;
  showConfirmModal.value = false;
  tabToCloseId.value = null;
  const tab = tabs.value.find(t => t.id === id);
  if (!tab || tab.busy) return;
  tab.connectionVersion = (tab.connectionVersion ?? 0) + 1;
  tab.busy = true;
  try {
    if (tab.connected) await ModbusService.disconnect(tab.id);
    tab.connected = false;
    handleCloseTab(tab.id);
  } catch (err) {
    logFailure(tab, 'Disconnect', err);
    tab.statusMessage = `[${tab.title}] Disconnect failed: ${err instanceof Error ? err.message : err}`;
    tab.statusType = 'error';
  } finally {
    tab.busy = false;
  }
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
  const operationStarted = Date.now();
  const tab = activeTab.value;
  if (!tab || tab.busy) return;
  tab.connectionVersion = (tab.connectionVersion ?? 0) + 1;
  tab.busy = true;
  try {
    if (tab.connected) {
      try {
        broadcastLog(tab, 'TX', `Disconnect Req`, `Session: ${tab.title}`);
        await ModbusService.disconnect(tab.id);
        tab.connected = false;
        tab.isPolling = false;
        tab.autoIncrement = false;
        clearTabTimers(tab.id);
        tab.statusMessage = `[${tab.title}] Disconnected.`;
        tab.statusType = 'info';
        broadcastLog(tab, 'RX', `Disconnected OK`, `Status: Closed`, false, operationStarted);
      } catch (err: any) {
        logFailure(tab, 'Disconnect', err, operationStarted);
        tab.statusMessage = `[${tab.title}] Disconnect failed: ${err?.message || err}`;
        tab.statusType = 'error';
      }
    } else {
      try {
        const targetStr = tab.connection.protocol === 'RTU'
          ? `${tab.connection.serialPort} (${tab.connection.baudRate})`
          : `${tab.connection.ip}:${tab.connection.port}`;
        broadcastLog(tab, 'TX', `${tab.connection.role} Connect (${tab.connection.protocol})`, targetStr);
        const msg = await ModbusService.connect(
          tab.id,
          tab.connection,
          tab.config.unitId,
          tab.title,
        );
        tab.connected = true;
        tab.statusMessage = `[${tab.title}] ${msg}`;
        tab.statusType = 'success';
        broadcastLog(tab, 'RX', `${tab.connection.role} Connected OK`, `Status: Active`, false, operationStarted);
      } catch (err: any) {
        logFailure(tab, 'Connect', err, operationStarted);
        tab.statusMessage = `[${tab.title}] Connection failed: ${err?.message || err}`;
        tab.statusType = 'error';
      }
    }
  } finally {
    tab.busy = false;
  }
}

async function handleFillRandom() {
  const operationStarted = Date.now();
  const tab = activeTab.value;
  if (!tab || !tab.connected || tab.busy) return;
  const isCurrent = captureTabRequest(tab);
  try {
    broadcastLog(tab, 'TX', `Random Fill Req`, `Start: ${tab.config.startAddress}, Count: ${tab.config.count}`);
    const updated = generateRandomRegisters(
      tab.config.startAddress,
      tab.config.count,
      tab.config.dataType,
      tab.config.byteOrder,
      tab.config.functionCode
    );
    const confirmed: Record<number, number> = {};
    for (const [addrStr, val] of Object.entries(updated)) {
      if (!isCurrent()) return;
      const address = Number(addrStr);
      await writeTabRegister(tab, address, val);
      if (!isCurrent()) return;
      confirmed[address] = val;
      tab.values[address] = val;
      tab.valueVersion = (tab.valueVersion ?? 0) + 1;
    }
    tab.values = { ...tab.values, ...confirmed };
    tab.statusMessage = `[${tab.title}] Random fill OK.`;
    tab.statusType = 'success';
    broadcastLog(tab, 'RX', `Random Fill Resp OK`, `Updated ${tab.config.count} registers`, false, operationStarted);
  } catch (err) {
    if (!isCurrent()) return;
    console.error('Failed fill random:', err);
    logFailure(tab, 'Random fill', err, operationStarted);
    tab.statusMessage = `[${tab.title}] Random fill failed: ${err instanceof Error ? err.message : err}`;
    tab.statusType = 'error';
  }
}

function handleToggleAutoIncrement() {
  if (!activeTab.value || !activeTab.value.connected || activeTab.value.busy) return;
  activeTab.value.autoIncrement = !activeTab.value.autoIncrement;
}

function handleTogglePoll() {
  if (!activeTab.value || !activeTab.value.connected || activeTab.value.busy) return;
  activeTab.value.isPolling = !activeTab.value.isPolling;
}

async function handleReadOnce() {
  const operationStarted = Date.now();
  const tab = activeTab.value;
  if (!tab || !tab.connected || tab.busy) return;
  const isCurrent = captureTabRequest(tab);
  broadcastLog(tab, 'TX', `Read (${tab.config.functionCode}) Req`, `Unit: ${tab.config.unitId}, Start: ${tab.config.startAddress}, Count: ${tab.config.count}`);

  try {
    const fetched = await readTabRegisters(tab);
    if (!isCurrent()) return;
    if (Object.keys(fetched).length > 0) {
      tab.values = { ...tab.values, ...fetched };
    }
    tab.statusMessage = `[${tab.title}] Read OK (${Object.keys(fetched).length} values).`;
    tab.statusType = 'success';
    broadcastLog(tab, 'RX', `Read (${tab.config.functionCode}) Resp`, `Unit: ${tab.config.unitId}, Start: ${tab.config.startAddress}, Count: ${Object.keys(fetched).length}`, false, operationStarted);
  } catch (err) {
    if (!isCurrent()) return;
    console.error('Read failed:', err);
    tab.statusMessage = `[${tab.title}] Read failed: ${err instanceof Error ? err.message : err}`;
    tab.statusType = 'error';
    broadcastLog(tab, 'RX', `Read (${tab.config.functionCode}) Failed`, String(err), false, operationStarted);
  }
}

async function handleUpdateCell(address: number, value: number) {
  const operationStarted = Date.now();
  const tab = activeTab.value;
  if (!tab || !tab.connected || tab.busy || (tab.connection.role === 'Master' && ['0x02', '0x04'].includes(tab.config.functionCode))) return;
  const isCurrent = captureTabRequest(tab);
  broadcastLog(tab, 'TX', `Write Register (Addr: ${address})`, `Val: ${value}`);
  try {
    await writeTabRegister(tab, address, value);
    if (!isCurrent()) return;
    tab.values[address] = value;
    tab.valueVersion = (tab.valueVersion ?? 0) + 1;
    tab.statusMessage = `[${tab.title}] Write OK (${address}).`;
    tab.statusType = 'success';
    broadcastLog(tab, 'RX', `Write Register Resp OK`, `Unit: ${tab.config.unitId}, Addr: ${address}, Value: ${value}`, false, operationStarted);
  } catch (err: any) {
    if (!isCurrent()) return;
    console.error('Failed to write register:', err);
    logFailure(tab, 'Write', err, operationStarted);
    tab.statusMessage = `[${tab.title}] Write failed: ${err?.message || err}`;
    tab.statusType = 'error';
  }
}

async function handleUpdateCellPair(address1: number, value1: number, address2: number, value2: number) {
  const operationStarted = Date.now();
  const tab = activeTab.value;
  if (!tab || !tab.connected || tab.busy || (tab.connection.role === 'Master' && ['0x02', '0x04'].includes(tab.config.functionCode))) return;
  const isCurrent = captureTabRequest(tab);
  broadcastLog(tab, 'TX', `Write Register Pair (Addr: ${address1}-${address2})`, `Word1: ${value1}, Word2: ${value2}`);
  try {
    await ModbusService.writeRegisters(
      tab.id,
      address1,
      [value1, value2],
      tab.config.functionCode,
      tab.config.unitId,
      tab.connection.timeoutMs,
      tab.connection.retries
    );
    if (!isCurrent()) return;
    tab.values[address1] = value1;
    tab.values[address2] = value2;
    tab.valueVersion = (tab.valueVersion ?? 0) + 1;
    tab.statusMessage = `[${tab.title}] Write OK (${address1}-${address2}).`;
    tab.statusType = 'success';
    broadcastLog(tab, 'RX', `Write Register Pair Resp OK`, `Unit: ${tab.config.unitId}, Addr: ${address1}-${address2}, Values: ${value1}, ${value2}`, false, operationStarted);
  } catch (err: any) {
    if (!isCurrent()) return;
    console.error('Failed to write register pair:', err);
    logFailure(tab, 'Write', err, operationStarted);
    tab.statusMessage = `[${tab.title}] Write failed: ${err?.message || err}`;
    tab.statusType = 'error';
  }
}

function handleSaveSettings(newConnection: ConnectionConfig) {
  if (!activeTab.value || activeTab.value.connected || activeTab.value.busy) return;
  activeTab.value.connection = newConnection;
}


</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-gray-100 font-sans select-none overflow-hidden">
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
      :session-id="activeTab.id"
      :role="activeTab.connection.role"
      :protocol="activeTab.connection.protocol"
      :ip="activeTab.connection.ip"
      :port="activeTab.connection.port"
      :serialPort="activeTab.connection.serialPort"
      :baudRate="activeTab.connection.baudRate"
      :connected="activeTab.connected"
      :loading="activeTab.busy"
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
      :busy="activeTab.busy"
      :role="activeTab.connection.role"
    />

    <DataGrid
      v-if="activeTab"
      :key="activeTab.id"
      :writable="activeTab.connected && !activeTab.busy && (activeTab.connection.role === 'Slave' || !['0x02', '0x04'].includes(activeTab.config.functionCode))"
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
      :devices="tabs.map(tab => ({ id: tab.id, title: tab.title }))"
      @clear="device => tabs.forEach(tab => { if (!device || tab.id === device) tab.logs = []; })"
      @close="showLogPanel = false"
    />

    <StatusBar
      v-if="activeTab"
      :message="activeTab.statusMessage"
      :type="activeTab.statusType"
      :logCount="allLogs.length"
      :showLogPanel="showLogPanel"
      :language="language"
      @toggle-logs="showLogPanel = !showLogPanel"
      @toggle-language="toggleLanguage"
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
      :title="$t('modal.closeConnection')"
      :message="$t('modal.closeConnectionMsg')"
      @close="showConfirmModal = false"
      @confirm="handleConfirmCloseTab"
    />
  </div>
</template>
