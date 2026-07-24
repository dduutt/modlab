# Modlab Requirements Re-Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-align Modlab codebase with Master vs. Slave role distinct workflows, format-based value auto-increment, and persistence reset policies.

**Architecture:** Update `modbusFormatter.ts`, `Toolbar.vue`, `ConfigRow.vue`, `DataGrid.vue`, and `App.vue`.

**Tech Stack:** Vue 3, TypeScript, Tailwind CSS, pnpm.

## Global Constraints
- Use `pnpm` for all build and verification commands.
- Preserve existing component props interfaces.

---

### Task 1: Format-Based Value Increment in `modbusFormatter.ts`

**Files:**
- Modify: `src/utils/modbusFormatter.ts`

**Interfaces:**
- Produces: `incrementFormattedValue(raw1: number, format: string, dataType: string, byteOrder: string, raw2?: number): { word1: number; word2?: number }`

- [ ] **Step 1: Implement `incrementFormattedValue` in `modbusFormatter.ts`**

```ts
export function incrementFormattedValue(
  raw1: number,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  raw2?: number
): { word1: number; word2?: number } {
  const currentStr = formatRegisterValue(raw1, format, dataType, byteOrder, raw2);
  const isCoil = format === 'Dec' && (dataType === 'Coil'); // fallback
  
  if (dataType === 'Float32') {
    const currentVal = parseFloat(currentStr) || 0;
    const nextVal = (currentVal + 1).toFixed(4);
    const parsed = parseFormattedRegisterValue(nextVal, format, dataType, byteOrder);
    if (typeof parsed === 'object') {
      return { word1: parsed.word1, word2: parsed.word2 };
    }
  }

  if (dataType === 'Int32' || dataType === 'UInt32') {
    const currentVal = parseInt(currentStr, format === 'Hex' ? 16 : 10) || 0;
    const nextVal = (currentVal + 1).toString();
    const parsed = parseFormattedRegisterValue(nextVal, format, dataType, byteOrder);
    if (typeof parsed === 'object') {
      return { word1: parsed.word1, word2: parsed.word2 };
    }
  }

  if (dataType === 'Int16') {
    const currentVal = parseInt(currentStr, format === 'Hex' ? 16 : 10) || 0;
    const nextVal = currentVal + 1;
    const parsed = parseFormattedRegisterValue(nextVal.toString(), format, dataType, byteOrder);
    return { word1: typeof parsed === 'number' ? parsed & 0xffff : 0 };
  }

  // Default UInt16 or Coil
  const currentVal = parseInt(currentStr, format === 'Hex' ? 16 : 10) || 0;
  const nextVal = currentVal + 1;
  const parsed = parseFormattedRegisterValue(nextVal.toString(), format, dataType, byteOrder);
  return { word1: typeof parsed === 'number' ? parsed & 0xffff : 0 };
}
```

- [ ] **Step 2: Run verification with pnpm run build**

Run: `pnpm run build`
Expected: PASS with 0 errors

---

### Task 2: Distinct Master & Slave Toolbars in `Toolbar.vue`

**Files:**
- Modify: `src/components/Toolbar.vue`

**Interfaces:**
- Consumes: `role`, `connected`, `functionCode`, `autoIncrement`
- Emits: `toggle-connect`, `open-settings`, `toggle-auto-increment`, `fill-random`, `toggle-poll`, `read-once`

- [ ] **Step 1: Update `Toolbar.vue` template & props**

```vue
<script setup lang="ts">
import { Play, Square, Settings, Shuffle, RefreshCw, Repeat } from '@lucide/vue';

const props = defineProps<{
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
  loading?: boolean;
  functionCode?: string;
  autoIncrement?: boolean;
  isPolling?: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connect'): void;
  (e: 'open-settings'): void;
  (e: 'toggle-auto-increment'): void;
  (e: 'fill-random'): void;
  (e: 'toggle-poll'): void;
  (e: 'read-once'): void;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center justify-between px-4 py-2 bg-white border-b border-gray-200 select-none text-xs min-h-[42px] gap-2">
    <!-- Left: Status & Connection Info -->
    <div class="flex items-center gap-2.5">
      <span class="font-semibold text-gray-900 text-sm">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-xs text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <!-- Connect / Listen Button -->
      <button
        @click="emit('toggle-connect')"
        :disabled="loading"
        :class="[
          'flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition cursor-pointer',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100'
            : 'bg-blue-600 text-white border-blue-600 hover:bg-blue-700 shadow-2xs'
        ]"
      >
        <component :is="connected ? Square : Play" class="w-3.5 h-3.5" />
        {{ role === 'Slave' ? (connected ? 'Stop' : 'Listen') : (connected ? 'Disconnect' : 'Connect') }}
      </button>

      <!-- Settings Button -->
      <button
        @click="!connected && emit('open-settings')"
        :disabled="connected"
        :title="connected ? 'Disconnect to change settings' : 'Settings'"
        :class="[
          'flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition',
          connected
            ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50 cursor-pointer'
        ]"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        Settings
      </button>
    </div>

    <!-- Right: Mode-Specific Controls -->
    <div class="flex items-center gap-2">
      <!-- SLAVE MODE CONTROLS -->
      <template v-if="role === 'Slave'">
        <!-- Auto Increment Toggle -->
        <button
          @click="emit('toggle-auto-increment')"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition cursor-pointer',
            autoIncrement
              ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold'
              : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50'
          ]"
          title="Auto Increment values by 1 every Interval"
        >
          <Repeat class="w-3.5 h-3.5 text-emerald-500" />
          Auto Increment
        </button>

        <!-- Random Action Button -->
        <button
          @click="emit('fill-random')"
          class="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer border border-gray-200"
          title="Generate random values once"
        >
          <Shuffle class="w-3.5 h-3.5 text-gray-500" />
          Random
        </button>
      </template>

      <!-- MASTER MODE CONTROLS -->
      <template v-else>
        <!-- Poll Toggle -->
        <button
          @click="emit('toggle-poll')"
          :disabled="!connected"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : isPolling
                ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold cursor-pointer'
                : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50 cursor-pointer'
          ]"
          title="Poll slave continuously every Interval"
        >
          <RefreshCw :class="['w-3.5 h-3.5 text-emerald-500', isPolling ? 'animate-spin' : '']" />
          Poll
        </button>

        <!-- Read Action Button -->
        <button
          @click="connected && emit('read-once')"
          :disabled="!connected"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200 border-gray-200 cursor-pointer'
          ]"
          title="Read values from slave once"
        >
          Read
        </button>
      </template>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Run verification with pnpm run build**

Run: `pnpm run build`
Expected: PASS with 0 errors

---

### Task 3: Refactor `App.vue` Master & Slave Handlers & Persistence

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add `autoIncrement` & `isPolling` fields to `SessionTab` interface and update handlers**

In `App.vue`:
- Implement `handleToggleAutoIncrement()`:
  Interval timer that calls `incrementFormattedValue` for each register/pair according to active format & data type.
- Implement `handleTogglePoll()`:
  Interval timer that calls `ModbusService.readRegisters()`.
- Implement `handleReadOnce()`:
  Single read call to `ModbusService.readRegisters()`.

- [ ] **Step 2: Run verification with pnpm run build**

Run: `pnpm run build`
Expected: PASS with 0 errors

---

### Task 4: Complete System Build Verification

- [ ] **Step 1: Run `pnpm run build`**

Run: `pnpm run build`
Expected: PASS with 0 errors, output generated in `dist/`.
