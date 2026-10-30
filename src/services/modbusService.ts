import { Channel, invoke } from '@tauri-apps/api/core';

export type FsmState =
  | 'Disconnected'
  | 'Connecting'
  | 'Connected'
  | 'Reconnecting'
  | { Error: string };

export interface ConnectionConfig {
  protocol: 'TCP' | 'RTU';
  ip: string;
  port: number;
  serialPort: string;
  baudRate: number;
  dataBits: number;
  stopBits: number;
  parity: string;
  role: 'Master' | 'Slave';
  unitId: number;
  functionCode: string;
  startAddress: number;
  count: number;
  timeoutMs?: number;
}

export interface WritePayload {
  address: number;
  value: number;
}

export type SessionCommand =
  | { type: 'Connect'; payload: ConnectionConfig }
  | { type: 'Disconnect' }
  | { type: 'UpdateConfig'; payload: ConnectionConfig }
  | { type: 'StartPolling'; payload: number }
  | { type: 'StopPolling' }
  | { type: 'ReadOnce' }
  | { type: 'WriteRegister'; payload: WritePayload }
  | { type: 'WriteRegistersBatch'; payload: Record<number, number> };

export type SessionEvent =
  | { type: 'StateChanged'; payload: FsmState }
  | { type: 'DataUpdated'; payload: Record<number, number> }
  | { type: 'PollError'; payload: string };

export interface TcpClientInfo {
  id: number;
  address: string;
  connectedAt: number;
  lastRequestAt: number | null;
}

export class ModbusService {
  static async spawnSession(
    sessionId: string,
    onEvent: (event: SessionEvent) => void
  ): Promise<void> {
    const channel = new Channel<SessionEvent>();
    channel.onmessage = (event) => {
      onEvent(event);
    };
    await invoke('spawn_session_native', { sessionId, onEvent: channel });
  }

  static async dispatchCommand(sessionId: string, command: SessionCommand): Promise<void> {
    await invoke('dispatch_session_command', { sessionId, command });
  }

  static async killSession(sessionId: string): Promise<void> {
    await invoke('kill_session_native', { sessionId });
  }

  static async listSerialPorts(): Promise<string[]> {
    return await invoke<string[]>('list_serial_ports_native');
  }

  static async listTcpClients(sessionId: string): Promise<TcpClientInfo[]> {
    return await invoke<TcpClientInfo[]>('list_tcp_clients_native', { sessionId });
  }

  static async connect(sessionId: string, config: any, unitId: number, sessionTitle?: string): Promise<string> {
    return await invoke<string>('connect_modbus_native', { sessionId, sessionTitle, config, unitId });
  }

  static async disconnect(sessionId: string): Promise<string> {
    return await invoke<string>('disconnect_modbus_native', { sessionId });
  }

  static async readRegisters(
    sessionId: string,
    start: number,
    count: number,
    functionCode: string,
    unitId: number,
    timeoutMs: number,
    retries: number,
  ): Promise<Record<number, number>> {
    return await invoke<Record<number, number>>('read_registers_native', {
      sessionId, start, count, functionCode, unitId, timeoutMs, retries,
    });
  }

  static async writeRegister(
    sessionId: string,
    address: number,
    value: number,
    functionCode: string,
    unitId: number,
    timeoutMs: number,
    retries: number,
    role?: string,
  ): Promise<void> {
    await invoke('write_register_native', {
      sessionId, address, value, functionCode, unitId, timeoutMs, retries, role,
    });
  }

  static async writeRegisters(
    sessionId: string,
    start: number,
    values: number[],
    functionCode: string,
    unitId: number,
    timeoutMs: number,
    retries: number,
    role?: string,
  ): Promise<void> {
    await invoke('write_registers_native', {
      sessionId, start, values, functionCode, unitId, timeoutMs, retries, role,
    });
  }
}
