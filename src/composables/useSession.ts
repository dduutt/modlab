import { ref, computed } from 'vue';
import {
  ModbusService,
  type FsmState,
  type ConnectionConfig as BackendConnectionConfig,
  type SessionCommand,
  type SessionEvent,
  type TcpClientInfo,
  type DataGenConfig,
} from '../services/modbusService';
import type { ConnectionConfig } from '../components/SettingsModal.vue';
import type { ModbusConfig } from '../components/ConfigRow.vue';

export function buildBackendConfig(
  conn: ConnectionConfig,
  cfg: ModbusConfig
): BackendConnectionConfig {
  return {
    protocol: conn.protocol,
    ip: conn.ip,
    port: conn.port,
    serialPort: conn.serialPort,
    baudRate: conn.baudRate,
    dataBits: conn.dataBits,
    stopBits: conn.stopBits,
    parity: conn.parity,
    role: conn.role,
    unitId: cfg.unitId,
    functionCode: cfg.functionCode,
    startAddress: cfg.startAddress,
    count: cfg.count,
    timeoutMs: conn.timeoutMs,
  };
}

export interface UseSessionOptions {
  onDataUpdated?: (data: Record<number, number>) => void;
  onStateChanged?: (state: FsmState) => void;
  onPollError?: (error: string) => void;
}

export function useSession(sessionId: string, options?: UseSessionOptions) {
  const fsmState = ref<FsmState>('Disconnected');
  const isSpawned = ref<boolean>(false);

  const isConnected = computed(() => fsmState.value === 'Connected');
  const isConnecting = computed(() => fsmState.value === 'Connecting' || fsmState.value === 'Reconnecting');
  const errorMessage = computed(() => {
    if (typeof fsmState.value === 'object' && 'Error' in fsmState.value) {
      return fsmState.value.Error;
    }
    return null;
  });

  async function ensureSpawned() {
    if (isSpawned.value) return;
    try {
      await ModbusService.spawnSession(sessionId, (event: SessionEvent) => {
        if (event.type === 'StateChanged') {
          fsmState.value = event.payload;
          options?.onStateChanged?.(event.payload);
        } else if (event.type === 'DataUpdated') {
          options?.onDataUpdated?.(event.payload);
        } else if (event.type === 'PollError') {
          options?.onPollError?.(event.payload);
        }
      });
      isSpawned.value = true;
    } catch (err) {
      console.error(`[useSession] spawn failed for ${sessionId}:`, err);
      fsmState.value = { Error: String(err) };
    }
  }

  async function dispatch(cmd: SessionCommand) {
    await ensureSpawned();
    await ModbusService.dispatchCommand(sessionId, cmd);
  }

  async function connect(conn: ConnectionConfig, cfg: ModbusConfig) {
    await ensureSpawned();
    const backendConfig = buildBackendConfig(conn, cfg);
    await dispatch({ type: 'Connect', payload: backendConfig });
  }

  async function disconnect() {
    await dispatch({ type: 'Disconnect' });
  }

  async function updateConfig(conn: ConnectionConfig, cfg: ModbusConfig) {
    const backendConfig = buildBackendConfig(conn, cfg);
    await dispatch({ type: 'UpdateConfig', payload: backendConfig });
  }

  async function startPolling(intervalMs: number) {
    await dispatch({ type: 'StartPolling', payload: intervalMs });
  }

  async function stopPolling() {
    await dispatch({ type: 'StopPolling' });
  }

  async function readOnce() {
    await dispatch({ type: 'ReadOnce' });
  }

  async function writeRegister(address: number, value: number) {
    await dispatch({ type: 'WriteRegister', payload: { address, value } });
  }

  async function writeRegistersBatch(batch: Record<number, number>) {
    await dispatch({ type: 'WriteRegistersBatch', payload: batch });
  }

  async function startAutoIncrement(intervalMs: number, config: DataGenConfig) {
    await dispatch({ type: 'StartAutoIncrement', payload: { intervalMs, config } });
  }

  async function stopAutoIncrement() {
    await dispatch({ type: 'StopAutoIncrement' });
  }

  async function fillRandom(config: DataGenConfig) {
    await dispatch({ type: 'FillRandom', payload: config });
  }

  async function listTcpClients(): Promise<TcpClientInfo[]> {
    return await ModbusService.listTcpClients(sessionId);
  }

  async function destroy() {
    try {
      await ModbusService.killSession(sessionId);
      isSpawned.value = false;
      fsmState.value = 'Disconnected';
    } catch (err) {
      console.warn(`[useSession] kill session ${sessionId} failed:`, err);
    }
  }

  return {
    sessionId,
    fsmState,
    isConnected,
    isConnecting,
    errorMessage,
    connect,
    disconnect,
    updateConfig,
    startPolling,
    stopPolling,
    readOnce,
    writeRegister,
    writeRegistersBatch,
    startAutoIncrement,
    stopAutoIncrement,
    fillRandom,
    listTcpClients,
    destroy,
  };
}
