import { invoke } from '@tauri-apps/api/core';
import type { ConnectionConfig } from '../components/SettingsModal.vue';

export class ModbusService {
  static async connect(sessionId: string, config: ConnectionConfig): Promise<string> {
    return await invoke<string>('connect_modbus_native', { sessionId, config });
  }

  static async disconnect(sessionId: string): Promise<string> {
    return await invoke<string>('disconnect_modbus_native', { sessionId });
  }

  static async readRegisters(sessionId: string, start: number, count: number): Promise<Record<number, number>> {
    return await invoke<Record<number, number>>('read_registers_native', { sessionId, start, count });
  }

  static async writeRegister(sessionId: string, address: number, value: number): Promise<void> {
    await invoke('write_register_native', { sessionId, address, value });
  }

  static async fillRandom(sessionId: string, start: number, count: number): Promise<Record<number, number>> {
    return await invoke<Record<number, number>>('fill_random_native', { sessionId, start, count });
  }

  static async fillIncrement(sessionId: string, start: number, count: number): Promise<Record<number, number>> {
    return await invoke<Record<number, number>>('fill_increment_native', { sessionId, start, count });
  }
}
