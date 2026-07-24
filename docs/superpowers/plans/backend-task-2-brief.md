# Backend Task 2 Brief: Frontend Service Bridge (`modbusService.ts`) & App Integration

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Tauri API: `@tauri-apps/api/core` (`invoke`)

## Instructions
1. Create `src/services/modbusService.ts` bridging Vue components with Tauri `invoke()` Rust backend commands (`connect_modbus`, `disconnect_modbus`, `read_registers`, `write_register`) with a fallback for browser simulation mode.
2. Update `src/App.vue` to use `ModbusService` when toggling connection or updating register cells.

Code for `src/services/modbusService.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionConfig } from '../components/SettingsModal.vue';

export class ModbusService {
  private static isTauri(): boolean {
    return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  }

  static async connect(config: ConnectionConfig): Promise<string> {
    if (this.isTauri()) {
      return await invoke<string>('connect_modbus', { config });
    }
    const target = config.protocol === 'TCP' ? `${config.ip}:${config.port}` : `${config.serialPort}`;
    return `Connected to ${target} (Browser Sim)`;
  }

  static async disconnect(): Promise<string> {
    if (this.isTauri()) {
      return await invoke<string>('disconnect_modbus');
    }
    return 'Disconnected (Browser Sim)';
  }

  static async readRegisters(start: number, count: number): Promise<Record<number, number>> {
    if (this.isTauri()) {
      return await invoke<Record<number, number>>('read_registers', { start, count });
    }
    return {};
  }

  static async writeRegister(address: number, value: number): Promise<void> {
    if (this.isTauri()) {
      await invoke('write_register', { address, value });
    }
  }
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/backend-task-2-report.md`.
