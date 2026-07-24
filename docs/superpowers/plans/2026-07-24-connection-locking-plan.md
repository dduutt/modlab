# Connection Parameter Locking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lock core parameters during active connection while allowing Master Unit ID and display formatting options to remain editable.

**Architecture:** Update `ConfigRow.vue`, `Toolbar.vue`, and `App.vue`.

**Tech Stack:** Vue 3, TypeScript, Tailwind CSS, pnpm.

## Global Constraints
- Must use `pnpm` for all commands.
- Keep existing props and emits intact.

---

### Task 1: Update `ConfigRow.vue` for Role-Based Parameter Locking

**Files:**
- Modify: `src/components/ConfigRow.vue`

- [ ] **Step 1: Add connected & role props and update template disabled states**

Replace `ConfigRow.vue` script & template:
```vue
<script setup lang="ts">
export interface ModbusConfig {
  unitId: number;
  functionCode: string;
  startAddress: number;
  count: number;
  dataType: string;
  format: string;
  byteOrder: string;
  interval: number;
  raw: boolean;
}

const props = defineProps<{
  config: ModbusConfig;
  connected?: boolean;
  role?: 'Slave' | 'Master';
}>();

const emit = defineEmits<{
  (e: 'update:config', value: ModbusConfig): void;
}>();

function updateField<K extends keyof ModbusConfig>(key: K, value: ModbusConfig[K]) {
  emit('update:config', { ...props.config, [key]: value });
}
</script>

<template>
  <div class="flex items-center justify-between gap-2 px-6 py-2.5 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[52px]">
    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Unit ID</label>
      <input
        type="number"
        :value="config.unitId"
        :disabled="connected && role === 'Slave'"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          (connected && role === 'Slave')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Function</label>
      <select
        :value="config.functionCode"
        :disabled="connected"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border rounded-full outline-none font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer border-gray-200'
        ]"
      >
        <option value="0x01">0x01 Coils</option>
        <option value="0x02">0x02 Inputs</option>
        <option value="0x03">0x03 Holding</option>
        <option value="0x04">0x04 InputRegs</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        :disabled="connected"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Count</label>
      <input
        type="number"
        :value="config.count"
        :disabled="connected"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Data Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 bg-white font-medium text-xs shadow-2xs cursor-pointer"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Format</label>
      <select
        :value="config.format"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border border-gray-200 rounded-full outline-none font-medium text-xs shadow-2xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Byte Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 bg-white font-medium text-xs shadow-2xs cursor-pointer"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Interval</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-20 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-1">Raw</label>
      <button
        type="button"
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-9 h-5 rounded-full p-0.5 transition-colors relative cursor-pointer my-1.5',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-4 h-4 rounded-full bg-white shadow-xs transition-transform',
            config.raw ? 'translate-x-4' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </div>
</template>
```

---

### Task 2: Update `Toolbar.vue` for Settings Button Disabling

**Files:**
- Modify: `src/components/Toolbar.vue`

- [ ] **Step 1: Disable Settings button when connected**

In `Toolbar.vue`:
```vue
      <button
        @click="!connected && emit('open-settings')"
        :disabled="connected"
        :title="connected ? 'Disconnect to change connection settings' : 'Connection Settings'"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
          connected
            ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50 cursor-pointer'
        ]"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        Settings
      </button>
```

---

### Task 3: Update `App.vue` Props Handoff to `ConfigRow`

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Pass connected & role props to ConfigRow tag in App.vue**

In `App.vue`:
```vue
    <ConfigRow
      v-if="activeTab"
      v-model:config="activeTab.config"
      :connected="activeTab.connected"
      :role="activeTab.connection.role"
    />
```

- [ ] **Step 2: Run verification with pnpm run build**
