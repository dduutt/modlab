import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { ConnectionConfig } from '../components/SettingsModal.vue';
import type { ModbusConfig } from '../components/ConfigRow.vue';
import type { LogEntry } from '../components/TrafficLog.vue';
import { appendLog } from '../utils/logging';
import { generateRandomRegisters, incrementFormattedValue } from '../utils/modbusFormatter';
import { useSession } from '../composables/useSession';

export interface SessionTab {
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
      title: 'Slave 1',
      connection: {
        role: 'Slave',
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
          connection: t.connection || {
            role: 'Slave',
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
          },
          connected: false,
          config: {
            unitId: t.config?.unitId ?? 1,
            functionCode: t.config?.functionCode ?? '0x03',
            startAddress: t.config?.startAddress ?? 0,
            count: t.config?.count ?? 40,
            dataType: t.config?.dataType ?? 'Int16',
            format: t.config?.format ?? 'Dec',
            byteOrder: t.config?.byteOrder ?? 'ABCD',
            interval: t.config?.interval ?? 1000,
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
  const autoIncTimers = new Map<string, ReturnType<typeof setInterval>>();

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

  function getOrCreateSession(tabId: string) {
    let session = sessions.get(tabId);
    if (!session) {
      session = useSession(tabId, {
        onStateChanged: (state) => {
          const target = tabs.value.find(t => t.id === tabId);
          if (!target) return;
          if (state === 'Connected') {
            target.connected = true;
            target.busy = false;
            target.statusType = 'success';
            target.statusMessage = `[${target.title}] Connected.`;
            addLog(target, 'RX', `${target.connection.role} Connected OK`, 'Status: Active');
          } else if (state === 'Disconnected') {
            target.connected = false;
            target.busy = false;
            target.isPolling = false;
            target.autoIncrement = false;
            stopAutoIncrement(tabId);
            target.statusType = 'info';
            target.statusMessage = `[${target.title}] Disconnected.`;
            addLog(target, 'RX', `${target.connection.role} Disconnected OK`, 'Status: Closed');
          } else if (typeof state === 'object' && 'Error' in state) {
            target.connected = false;
            target.busy = false;
            target.isPolling = false;
            target.autoIncrement = false;
            stopAutoIncrement(tabId);
            target.statusType = 'error';
            target.statusMessage = `[${target.title}] Error: ${state.Error}`;
            addLog(target, 'RX', `${target.connection.role} Error`, state.Error);
          }
        },
        onDataUpdated: (data) => {
          const target = tabs.value.find(t => t.id === tabId);
          if (!target) return;
          target.values = { ...target.values, ...data };
          if (target.statusType === 'error' && target.statusMessage.includes('failed')) {
            target.statusType = 'success';
            target.statusMessage = `[${target.title}] Communication restored.`;
            addLog(target, 'RX', 'Communication restored', '');
          }
        },
        onPollError: (err) => {
          const target = tabs.value.find(t => t.id === tabId);
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

  function stopAutoIncrement(tabId: string) {
    const timer = autoIncTimers.get(tabId);
    if (timer) {
      clearInterval(timer);
      autoIncTimers.delete(tabId);
    }
  }

  function startAutoIncrement(tabId: string) {
    stopAutoIncrement(tabId);
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || !tab.connected || ['0x02', '0x04'].includes(tab.config.functionCode)) return;

    const timer = setInterval(async () => {
      const currentTab = tabs.value.find(t => t.id === tabId);
      if (!currentTab || !currentTab.connected || !currentTab.autoIncrement || ['0x02', '0x04'].includes(currentTab.config.functionCode)) {
        stopAutoIncrement(tabId);
        return;
      }

      const session = getOrCreateSession(tabId);
      const cfg = currentTab.config;
      const isCoil = cfg.functionCode === '0x01' || cfg.functionCode === '0x02';
      const is32 =
        !isCoil && (cfg.dataType === 'Float32' || cfg.dataType === 'Int32' || cfg.dataType === 'UInt32');
      const step = is32 ? 2 : 1;
      const updatedBatch: Record<number, number> = {};

      for (let addr = cfg.startAddress; addr < cfg.startAddress + cfg.count; addr += step) {
        if (is32 && addr + 1 >= cfg.startAddress + cfg.count) break;
        const raw1 = currentTab.values[addr] ?? 0;
        const raw2 = is32 ? (currentTab.values[addr + 1] ?? 0) : undefined;
        const inc = incrementFormattedValue(
          raw1,
          cfg.format,
          isCoil ? 'Coil' : cfg.dataType,
          cfg.byteOrder,
          raw2
        );
        updatedBatch[addr] = inc.word1;
        if (is32 && inc.word2 !== undefined && addr + 1 < cfg.startAddress + cfg.count) {
          updatedBatch[addr + 1] = inc.word2;
        }
      }

      try {
        await session.writeRegistersBatch(updatedBatch);
        currentTab.values = { ...currentTab.values, ...updatedBatch };
      } catch (err) {
        console.error(`[AutoIncrement] failed for ${tabId}:`, err);
      }
    }, tab.config.interval || 1000);

    autoIncTimers.set(tabId, timer);
  }

  async function toggleConnect(tabId: string) {
    const tab = tabs.value.find(t => t.id === tabId);
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
    const tab = tabs.value.find(t => t.id === tabId);
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

  function toggleAutoIncrement(tabId: string) {
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || !tab.connected || tab.busy) return;
    tab.autoIncrement = !tab.autoIncrement;
    if (tab.autoIncrement) {
      startAutoIncrement(tabId);
      tab.statusMessage = `[${tab.title}] Auto increment started.`;
    } else {
      stopAutoIncrement(tabId);
      tab.statusMessage = `[${tab.title}] Auto increment stopped.`;
    }
  }

  async function readOnce(tabId: string) {
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || !tab.connected || tab.busy) return;
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
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || ['0x02', '0x04'].includes(tab.config.functionCode)) return;
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
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || ['0x02', '0x04'].includes(tab.config.functionCode)) return;
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
    const tab = tabs.value.find(t => t.id === tabId);
    if (!tab || (!tab.connected && tab.connection.role !== 'Slave') || tab.busy) return;
    if (tab.isPolling || tab.autoIncrement || ['0x02', '0x04'].includes(tab.config.functionCode)) return;
    const session = getOrCreateSession(tabId);
    const started = Date.now();
    addLog(
      tab,
      'TX',
      `Random Fill Req`,
      `Start: ${tab.config.startAddress}, Count: ${tab.config.count}`
    );
    try {
      const updated = generateRandomRegisters(
        tab.config.startAddress,
        tab.config.count,
        tab.config.dataType,
        tab.config.byteOrder,
        tab.config.functionCode
      );
      await session.writeRegistersBatch(updated);
      tab.values = { ...tab.values, ...updated };
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
    const tab = tabs.value.find(t => t.id === tabId);
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
    }
  }

  function updateConnection(tabId: string, newConnection: ConnectionConfig, newTitle?: string) {
    const tab = tabs.value.find(t => t.id === tabId);
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
