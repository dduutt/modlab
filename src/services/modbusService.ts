import { invoke } from '@tauri-apps/api/core';
import type { ConnectionConfig } from '../components/SettingsModal.vue';

export interface TcpClientInfo {
  id: number;
  address: string;
  connectedAt: number;
  lastRequestAt: number | null;
}

export class ModbusService {
  static async listTcpClients(sessionId: string): Promise<TcpClientInfo[]> {
    return invoke<TcpClientInfo[]>('list_tcp_clients_native', { sessionId });
  }

  static async listSerialPorts(): Promise<string[]> {
    return await invoke<string[]>('list_serial_ports_native');
  }

  static async connect(sessionId: string, config: ConnectionConfig, unitId: number, sessionTitle?: string): Promise<string> {
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
  ): Promise<void> {
    await invoke('write_register_native', {
      sessionId, address, value, functionCode, unitId, timeoutMs, retries,
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
  ): Promise<void> {
    await invoke('write_registers_native', {
      sessionId, start, values, functionCode, unitId, timeoutMs, retries,
    });
  }
}
