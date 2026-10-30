import { ModbusService } from '../services/modbusService';
import { appendLog } from './logging';
import { generateRandomRegisters, incrementFormattedValue } from './modbusFormatter';

declare const activeTab: any;
declare const tabToCloseId: any;
declare const showConfirmModal: any;
declare const tabs: any;
declare const pollingInFlight: Set<string>;
declare const incrementInFlight: Set<string>;
declare const tabTimers: Map<string, any>;

function trimLogs(session: { logs: any[] }) {
  session.logs = session.logs.slice(0, 2000);
}

function broadcastLog(
  session: any,
  direction: 'TX' | 'RX',
  message: string,
  bytes: string,
  detail = false,
  startedAt?: number
) {
  const timestampMs = Date.now();
  const level = /failed/i.test(message) ? 'error' : /restored/i.test(message) ? 'success' : 'info';
  const logEntry = {
    origin: 'frontend',
    id: crypto.randomUUID(),
    sessionId: session.id,
    sessionTitle: session.title,
    direction,
    message,
    bytes,
    timestampMs,
    durationMs: startedAt === undefined ? undefined : Math.max(0, timestampMs - startedAt),
    timestamp:
      new Date(timestampMs).toLocaleTimeString('en-GB', { hour12: false }) +
      '.' +
      String(timestampMs % 1000).padStart(3, '0'),
    kind: 'operation' as const,
    level: level as 'error' | 'success' | 'info',
    detail: (detail || direction === 'TX') && level !== 'error',
  };
  if (level !== 'error' || !/read|write|connect|disconnect/i.test(message)) return;
  session.logs = appendLog(session.logs, {
    ...logEntry,
    time: logEntry.timestamp,
    deviceName: session.title,
  });
  trimLogs(session);
}

function logFailure(session: any, operation: string, error: unknown, startedAt?: number) {
  broadcastLog(
    session,
    'RX',
    `${operation} Failed`,
    error instanceof Error ? error.message : String(error),
    false,
    startedAt
  );
}

function captureTabRequest(tab: any) {
  const connection = tab.connection;
  const version = tab.connectionVersion;
  const { unitId, functionCode, startAddress, count } = tab.config;
  return () =>
    (tab.connection.role === 'Slave' || tab.connected) &&
    !tab.busy &&
    tab.connection === connection &&
    tab.connectionVersion === version &&
    tab.config.unitId === unitId &&
    tab.config.functionCode === functionCode &&
    tab.config.startAddress === startAddress &&
    tab.config.count === count;
}

function readTabRegisters(tab: any) {
  return ModbusService.readRegisters(
    tab.id,
    tab.config.startAddress,
    tab.config.count,
    tab.config.functionCode,
    tab.config.unitId,
    tab.connection.timeoutMs,
    tab.connection.retries
  );
}

function writeTabRegister(tab: any, address: number, value: number) {
  return ModbusService.writeRegister(
    tab.id,
    address,
    value,
    tab.config.functionCode,
    tab.config.unitId,
    tab.connection.timeoutMs,
    tab.connection.retries,
    tab.connection.role
  );
}

function clearTabTimers(tabId: string) {
  const existing = tabTimers.get(tabId);
  if (existing) {
    if (existing.pollTimer) clearInterval(existing.pollTimer);
    if (existing.autoIncTimer) clearInterval(existing.autoIncTimer);
    tabTimers.delete(tabId);
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
        const targetStr =
          tab.connection.protocol === 'RTU'
            ? `${tab.connection.serialPort} (${tab.connection.baudRate})`
            : `${tab.connection.ip}:${tab.connection.port}`;
        broadcastLog(tab, 'TX', `${tab.connection.role} Connect (${tab.connection.protocol})`, targetStr);
        const msg = await ModbusService.connect(
          tab.id,
          tab.connection,
          tab.config.unitId,
          tab.title
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
  if (!tab || (!tab.connected && tab.connection.role !== 'Slave') || tab.busy) return;
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
  if (!tab || tab.busy || ['0x02', '0x04'].includes(tab.config.functionCode) || (tab.connection.role === 'Master' && (!tab.connected || tab.isPolling))) return;
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
  if (!tab || tab.busy || ['0x02', '0x04'].includes(tab.config.functionCode) || (tab.connection.role === 'Master' && (!tab.connected || tab.isPolling))) return;
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
      tab.connection.retries,
      tab.connection.role
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

function handleCloseTab(id: string) {
  void id;
}

async function handleConfirmCloseTab() {
  const id = tabToCloseId.value;
  showConfirmModal.value = false;
  tabToCloseId.value = null;
  const tab = tabs.value.find((t: any) => t.id === id);
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

function updateTabTimers(tab: any) {
  const existing = tabTimers.get(tab.id);
  if (existing) {
    if (existing.pollTimer) clearInterval(existing.pollTimer);
    if (existing.autoIncTimer) clearInterval(existing.autoIncTimer);
    tabTimers.delete(tab.id);
  }
  const timers: any = {};

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
          logFailure(tab, 'Slave sync', err);
          tab.statusMessage = `[${tab.title}] Sync failed: ${err instanceof Error ? err.message : err}`;
          tab.statusType = 'error';
        }
      } finally {
        pollingInFlight.delete(tab.id);
      }
    }, tab.config.interval || 1000);
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

export {
  trimLogs,
  broadcastLog,
  logFailure,
  captureTabRequest,
  readTabRegisters,
  writeTabRegister,
  clearTabTimers,
  handleToggleConnect,
  handleFillRandom,
  handleReadOnce,
  handleUpdateCell,
  handleUpdateCellPair,
  handleCloseTab,
  handleConfirmCloseTab,
  updateTabTimers,
};
