import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { ConnectionConfig } from '../components/SettingsModal.vue';
import type { ModbusConfig } from '../components/ConfigRow.vue';
import type { LogEntry } from '../components/TrafficLog.vue';
import { appendLog } from '../utils/logging';
import { useSession } from '../composables/useSession';

export interface SessionTab {
  id: string;
  title: string;
  connection: ConnectionConfig;
  connected: boolean;
  busy?: boolean;
  config: ModbusConfig;
  values: Record<number, number>;
  logs: LogEntry[];
  statusMessage: string;
  statusType: 'error' | 'success' | 'info';
  autoIncrement?: boolean;
  isPolling?: boolean;
}

const STORAGE_KEY = 'modlab_session_configs_v1';

export function createDefaultConnection(role: 'Slave' | 'Master' = 'Slave'): ConnectionConfig {
  return {
    role,
    protocol: 'TCP',
    ip: '127.0.0.1',
    port: 502,
    serialPort: 'COM1',
    baudRate: 9600,
    dataBits: 8,
    stopBits: 1,
    parity: 'None',
    timeoutMs: 1000,
    retries: 3,
  };
}

export function createDefaultModbusConfig(): ModbusConfig {
  return {
    unitId: 1,
    functionCode: '0x03',
    startAddress: 0,
    count: 40,
    dataType: 'Int16',
    format: 'Dec',
    byteOrder: 'ABCD',
    interval: 1000,
    raw: false,
  };
}

function getDefaultTabs(): SessionTab[] {
  return [
    {
      id: 'slave-1',
      title: 'Slave 1',
      connection: createDefaultConnection('Slave'),
      connected: false,
      config: createDefaultModbusConfig(),
      values: {},
      logs: [],
      statusMessage: '[Slave 1] Disconnected.',
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
          connection: { ...createDefaultConnection('Slave'), ...t.connection },
          connected: false,
          config: {
            ...createDefaultModbusConfig(),
            ...t.config,
            raw: Boolean(t.config?.raw),
          },
          values: {},
          logs: [],
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

export const useSessionStore = defineStore('session', () => {
  const { loadedTabs, savedActiveId } = loadSavedState();
  const tabs = ref<SessionTab[]>(loadedTabs);
  const activeTabId = ref<string>(savedActiveId);

  const sessions = new Map<string, ReturnType<typeof useSession>>();

  // Auto-save configuration to localStorage
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

  function addLog(
    tab: SessionTab,
    direction: 'TX' | 'RX',
    message: string,
    bytes: string,
    detail = false,
    startedAt?: number
  ) {
    const timestampMs = Date.now();
    const level = /failed|error/i.test(message)
      ? 'error'
      : /restored|connected|ok/i.test(message)
      ? 'success'
      : 'info';

    const logEntry: LogEntry = {
      id: crypto.randomUUID(),
      sessionId: tab.id,
      deviceName: tab.title,
      direction,
      message,
      bytes,
      timestampMs,
      durationMs: startedAt === undefined ? undefined : Math.max(0, timestampMs - startedAt),
      time:
        new Date(timestampMs).toLocaleTimeString('en-GB', { hour12: false }) +
        '.' +
        String(timestampMs % 1000).padStart(3, '0'),
      kind: 'operation',
      level,
      detail: (detail || direction === 'TX') && level !== 'error',
      role: tab.connection.role,
      protocol: tab.connection.protocol,
    };

    tab.logs = appendLog(tab.logs, logEntry);
  }

  const findTab = (tabId: string) => tabs.value.find(t => t.id === tabId);
  const isReadonlyFc = (fc: string) => ['0x02', '0x04'].includes(fc);

  function resetRuntime(target: SessionTab, stopSession = true) {
    target.connected = false;
    target.busy = false;
    target.isPolling = false;
    target.autoIncrement = false;
    if (stopSession) stopAutoIncrement(target.id);
  }

  function getOrCreateSession(tabId: string) {
    let session = sessions.get(tabId);
    if (!session) {
      session = useSession(tabId, {
        onStateChanged: (state) => {
          const target = findTab(tabId);
          if (!target) return;
          if (state === 'Connected') {
            target.connected = true;
            target.busy = false;
            target.statusType = 'success';
            target.statusMessage = `[${target.title}] Connected.`;
            addLog(target, 'RX', `${target.connection.role} Connected OK`, 'Status: Active');
          } else if (state === 'Disconnected') {
            resetRuntime(target);
            target.statusType = 'info';
            target.statusMessage = `[${target.title}] Disconnected.`;
            addLog(target, 'RX', `${target.connection.role} Disconnected OK`, 'Status: Closed');
          } else if (typeof state === 'object' && 'Error' in state) {
            resetRuntime(target);
            target.statusType = 'error';
            target.statusMessage = `[${target.title}] Error: ${state.Error}`;
            addLog(target, 'RX', `${target.connection.role} Error`, state.Error);
          }
        },
        onDataUpdated: (data) => {
          const target = findTab(tabId);
          if (!target) return;
          target.values = { ...target.values, ...data };
          if (target.statusType === 'error' && target.statusMessage.includes('failed')) {
            target.statusType = 'success';
            target.statusMessage = `[${target.title}] Communication restored.`;
            addLog(target, 'RX', 'Communication restored', '');
          }
        },
        onPollError: (err) => {
          const target = findTab(tabId);
          if (!target) return;
          target.statusType = 'error';
          target.statusMessage = `[${target.title}] Operation failed: ${err}`;
          addLog(target, 'RX', 'Operation Failed', err);
        },
      });
      sessions.set(tabId, session);
    }
    return session;
  }

  function buildDataGenConfig(cfg: ModbusConfig) {
    return {
      startAddress: cfg.startAddress,
      count: cfg.count,
      functionCode: cfg.functionCode,
      dataType: cfg.dataType,
      byteOrder: cfg.byteOrder,
    };
  }

  async function stopAutoIncrement(tabId: string) {
    const session = sessions.get(tabId);
    if (session) {
      await session.stopAutoIncrement();
    }
  }

  async function startAutoIncrement(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || !tab.connected || isReadonlyFc(tab.config.functionCode)) return;
    const session = getOrCreateSession(tabId);
    await session.startAutoIncrement(tab.config.interval || 1000, buildDataGenConfig(tab.config));
  }

  async function toggleConnect(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || tab.busy) return;
    const session = getOrCreateSession(tabId);
    const started = Date.now();
    tab.busy = true;

    try {
      if (tab.connected) {
        addLog(tab, 'TX', 'Disconnect Req', `Session: ${tab.title}`);
        await session.disconnect();
      } else {
        const targetStr =
          tab.connection.protocol === 'RTU'
            ? `${tab.connection.serialPort} (${tab.connection.baudRate})`
            : `${tab.connection.ip}:${tab.connection.port}`;
        addLog(tab, 'TX', `${tab.connection.role} Connect (${tab.connection.protocol})`, targetStr);
        await session.connect(tab.connection, tab.config);
      }
    } catch (err: any) {
      tab.busy = false;
      tab.statusType = 'error';
      tab.statusMessage = `[${tab.title}] Connect failed: ${err?.message || err}`;
      addLog(tab, 'RX', 'Connect Failed', String(err), false, started);
    }
  }

  async function togglePolling(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || !tab.connected || tab.busy) return;
    const session = getOrCreateSession(tabId);
    if (tab.isPolling) {
      tab.isPolling = false;
      await session.stopPolling();
      tab.statusMessage = `[${tab.title}] Polling stopped.`;
      tab.statusType = 'info';
    } else {
      tab.isPolling = true;
      await session.startPolling(tab.config.interval || 1000);
      tab.statusMessage = `[${tab.title}] Polling started (${tab.config.interval}ms).`;
      tab.statusType = 'info';
    }
  }

  async function toggleAutoIncrement(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || !tab.connected || tab.busy) return;
    tab.autoIncrement = !tab.autoIncrement;
    if (tab.autoIncrement) {
      await startAutoIncrement(tabId);
      tab.statusMessage = `[${tab.title}] Auto increment started.`;
    } else {
      await stopAutoIncrement(tabId);
      tab.statusMessage = `[${tab.title}] Auto increment stopped.`;
    }
  }

  async function readOnce(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || !tab.connected || tab.busy || tab.isPolling) return;
    const session = getOrCreateSession(tabId);
    addLog(
      tab,
      'TX',
      `Read (${tab.config.functionCode}) Req`,
      `Unit: ${tab.config.unitId}, Start: ${tab.config.startAddress}, Count: ${tab.config.count}`
    );
    await session.readOnce();
  }

  async function writeCell(tabId: string, address: number, value: number) {
    const tab = findTab(tabId);
    if (!tab || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || isReadonlyFc(tab.config.functionCode)) return;
    const session = getOrCreateSession(tabId);
    const started = Date.now();
    addLog(tab, 'TX', `Write Register (Addr: ${address})`, `Val: ${value}`);
    try {
      await session.writeRegister(address, value);
      tab.values[address] = value;
      tab.statusMessage = `[${tab.title}] Write OK (${address}).`;
      tab.statusType = 'success';
      addLog(tab, 'RX', `Write Register Resp OK`, `Addr: ${address}, Val: ${value}`, false, started);
    } catch (err: any) {
      tab.statusMessage = `[${tab.title}] Write failed: ${err?.message || err}`;
      tab.statusType = 'error';
      addLog(tab, 'RX', 'Write Failed', String(err), false, started);
    }
  }

  async function writeCellPair(
    tabId: string,
    addr1: number,
    val1: number,
    addr2: number,
    val2: number
  ) {
    const tab = findTab(tabId);
    if (!tab || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || isReadonlyFc(tab.config.functionCode)) return;
    const session = getOrCreateSession(tabId);
    const started = Date.now();
    addLog(
      tab,
      'TX',
      `Write Register Pair (Addr: ${addr1}-${addr2})`,
      `Word1: ${val1}, Word2: ${val2}`
    );
    try {
      await session.writeRegistersBatch({ [addr1]: val1, [addr2]: val2 });
      tab.values[addr1] = val1;
      tab.values[addr2] = val2;
      tab.statusMessage = `[${tab.title}] Write OK (${addr1}-${addr2}).`;
      tab.statusType = 'success';
      addLog(
        tab,
        'RX',
        `Write Register Pair Resp OK`,
        `Addrs: ${addr1}, ${addr2}`,
        false,
        started
      );
    } catch (err: any) {
      tab.statusMessage = `[${tab.title}] Write pair failed: ${err?.message || err}`;
      tab.statusType = 'error';
      addLog(tab, 'RX', 'Write Pair Failed', String(err), false, started);
    }
  }

  async function fillRandom(tabId: string) {
    const tab = findTab(tabId);
    if (!tab || (!tab.connected && tab.connection.role !== 'Slave') || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || isReadonlyFc(tab.config.functionCode)) return;
    const session = getOrCreateSession(tabId);
    const started = Date.now();
    addLog(
      tab,
      'TX',
      `Random Fill Req`,
      `Start: ${tab.config.startAddress}, Count: ${tab.config.count}`
    );
    try {
      await session.fillRandom(buildDataGenConfig(tab.config));
      tab.statusMessage = `[${tab.title}] Random fill OK.`;
      tab.statusType = 'success';
      addLog(
        tab,
        'RX',
        `Random Fill Resp OK`,
        `Updated ${tab.config.count} registers`,
        false,
        started
      );
    } catch (err: any) {
      tab.statusMessage = `[${tab.title}] Random fill failed: ${err?.message || err}`;
      tab.statusType = 'error';
      addLog(tab, 'RX', 'Random Fill Failed', String(err), false, started);
    }
  }

  function updateConfig(tabId: string, nextConfig: ModbusConfig) {
    const tab = findTab(tabId);
    if (!tab) return;
    const minInterval = tab.connection.protocol === 'RTU' ? 100 : 20;
    nextConfig.interval = Math.max(minInterval, nextConfig.interval);
    const oldFc = tab.config.functionCode;
    const oldUnit = tab.config.unitId;
    tab.config = nextConfig;

    if (oldFc !== nextConfig.functionCode || oldUnit !== nextConfig.unitId) {
      tab.values = {};
    }

    const session = sessions.get(tabId);
    if (session) {
      session.updateConfig(tab.connection, tab.config);
      if (tab.isPolling) {
        session.startPolling(nextConfig.interval || 1000);
      }
      if (tab.autoIncrement) {
        session.startAutoIncrement(nextConfig.interval || 1000, buildDataGenConfig(nextConfig));
      }
    }
  }

  function updateConnection(tabId: string, newConnection: ConnectionConfig, newTitle?: string) {
    const tab = findTab(tabId);
    if (!tab || tab.connected || tab.busy) return;
    tab.connection = { ...newConnection };
    if (newTitle && newTitle.trim()) {
      tab.title = newTitle.trim();
    }
  }

  function addTab(newConnection: ConnectionConfig, name?: string) {
    const role = newConnection.role || 'Slave';
    const deviceName = name?.trim() || (role === 'Master' ? 'Master 1' : 'Slave 1');
    const newId = `${role.toLowerCase()}-${Date.now()}`;
    tabs.value.push({
      id: newId,
      title: deviceName,
      connection: { ...newConnection },
      connected: false,
      config: createDefaultModbusConfig(),
      values: {},
      logs: [],
      statusMessage: `[${deviceName}] Idle.`,
      statusType: 'info',
    });
    activeTabId.value = newId;
  }

  async function closeTab(tabId: string) {
    if (tabs.value.length <= 1) return;
    stopAutoIncrement(tabId);
    const session = sessions.get(tabId);
    if (session) {
      try {
        await session.destroy();
      } catch (e) {
        console.warn('Destroy session failed:', e);
      }
      sessions.delete(tabId);
    }
    tabs.value = tabs.value.filter(t => t.id !== tabId);
    if (activeTabId.value === tabId) {
      activeTabId.value = tabs.value[0].id;
    }
  }

  function clearLogs(tabId?: string) {
    tabs.value.forEach(tab => {
      if (!tabId || tab.id === tabId) {
        tab.logs = [];
      }
    });
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    existingNames,
    allLogs,
    toggleConnect,
    togglePolling,
    toggleAutoIncrement,
    readOnce,
    writeCell,
    writeCellPair,
    fillRandom,
    updateConfig,
    updateConnection,
    addTab,
    closeTab,
    clearLogs,
    getOrCreateSession,
  };
});
