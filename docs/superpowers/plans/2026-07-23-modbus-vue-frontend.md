# Modbus Vue Tauri App Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the frontend design for a premium Modbus multi-slave/master Windows desktop application using Vue 3 + TypeScript + pnpm, matching the reference image style and enhancing visual details.

**Architecture:** 
A Vue 3 Single File Component (SFC) layout integrated into the existing Tauri + Vite environment. The app uses `<script setup lang="ts">` with modular components: `TabBar.vue`, `Toolbar.vue`, `ConfigRow.vue`, `DataGrid.vue`, and `StatusBar.vue`. State management is centralized in reactive Vue state (`ref`/`reactive`) per tab.

**Tech Stack:** Vue 3, TypeScript, Vite, pnpm, TailwindCSS + Vanilla CSS, Lucide icons (`@lucide/vue`), Tauri v2 API.

## Global Constraints

- Target Window Size: 1000x700.
- Package Manager: `pnpm` exclusively.
- Framework: Vue 3 (`<script setup lang="ts">`).
- Design Aesthetics: Clean desktop layout, soft rounded borders, crisp typography (Inter), responsive data grid, visual status indicators.

---

### Task 1: Tab Navigation Component (`TabBar.vue`)

**Files:**
- Create: `src/components/TabBar.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Props: `tabs: Array<{ id: string; title: string; active: boolean }>`
- Emits: `select-tab(id)`, `add-tab()`, `close-tab(id)`

- [ ] **Step 1: Create `src/components/TabBar.vue`**

```vue
<script setup lang="ts">
import { Plus, X } from 'lucide-vue-next';

interface Tab {
  id: string;
  title: string;
}

defineProps<{
  tabs: Tab[];
  activeTabId: string;
}>();

const emit = defineEmits<{
  (e: 'select-tab', id: string): void;
  (e: 'add-tab'): void;
  (e: 'close-tab', id: string): void;
}>();
</script>

<template>
  <div class="flex items-center bg-white px-4 pt-2 border-b border-gray-200 gap-2 select-none">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      :class="[
        'flex items-center gap-2 px-4 py-2 border rounded-t-lg text-sm font-medium transition-all cursor-pointer group',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-sm -mb-px z-10'
          : 'bg-gray-50 border-gray-200 text-gray-500 hover:bg-gray-100 hover:text-gray-700'
      ]"
    >
      <span
        :class="[
          'w-2 h-2 rounded-full',
          tab.id === activeTabId ? 'bg-blue-500' : 'bg-gray-300'
        ]"
      ></span>
      <span>{{ tab.title }}</span>
      <button
        v-if="tabs.length > 1"
        @click.stop="emit('close-tab', tab.id)"
        class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-gray-200 rounded transition"
      >
        <X class="w-3.5 h-3.5 text-gray-400 hover:text-gray-600" />
      </button>
    </div>

    <button
      @click="emit('add-tab')"
      class="p-1.5 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition"
      title="Add new session"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
```

- [ ] **Step 2: Verify in App.vue**

---

### Task 2: Action Toolbar Component (`Toolbar.vue`)

**Files:**
- Create: `src/components/Toolbar.vue`

**Interfaces:**
- Props: `role: 'Slave' | 'Master'`, `protocol: string`, `ip: string`, `port: number`, `connected: boolean`
- Emits: `toggle-connect()`, `open-settings()`, `fill-random()`, `fill-increment()`

- [ ] **Step 1: Create `src/components/Toolbar.vue`**

```vue
<script setup lang="ts">
import { Settings, Play, Square, Shuffle, TrendingUp } from 'lucide-vue-next';

defineProps<{
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connect'): void;
  (e: 'open-settings'): void;
  (e: 'fill-random'): void;
  (e: 'fill-increment'): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-3 bg-white border-b border-gray-200">
    <div class="flex items-center gap-4">
      <span class="font-semibold text-gray-900 text-base">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-sm text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <button
        @click="emit('toggle-connect')"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium border transition',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100'
            : 'bg-white text-gray-900 border-gray-300 hover:bg-gray-50'
        ]"
      >
        <component :is="connected ? Square : Play" class="w-4 h-4" />
        {{ connected ? 'Disconnect' : 'Connect' }}
      </button>

      <button
        @click="emit('open-settings')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-white text-gray-700 border border-gray-200 hover:bg-gray-50 transition"
      >
        <Settings class="w-4 h-4 text-gray-500" />
        Settings
      </button>
    </div>

    <div class="flex items-center gap-3">
      <button
        @click="emit('fill-random')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition"
      >
        <Shuffle class="w-3.5 h-3.5 text-gray-500" />
        Random
      </button>
      <button
        @click="emit('fill-increment')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition"
      >
        <TrendingUp class="w-3.5 h-3.5 text-gray-500" />
        Increment
      </button>
    </div>
  </div>
</template>
```

---

### Task 3: Configuration Panel Component (`ConfigRow.vue`)

**Files:**
- Create: `src/components/ConfigRow.vue`

**Interfaces:**
- Props: `config: ModbusConfig`
- Emits: `update:config(newConfig)`

- [ ] **Step 1: Create `src/components/ConfigRow.vue`**

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
}>();

const emit = defineEmits<{
  (e: 'update:config', value: ModbusConfig): void;
}>();

function updateField<K extends keyof ModbusConfig>(key: K, value: ModbusConfig[K]) {
  emit('update:config', { ...props.config, [key]: value });
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-4 px-4 py-3 bg-white border-b border-gray-200 text-sm">
    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Unit ID</label>
      <input
        type="number"
        :value="config.unitId"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        class="w-16 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Function</label>
      <select
        :value="config.functionCode"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="0x01">Read Coils (0x01)</option>
        <option value="0x02">Read Discrete Inputs (0x02)</option>
        <option value="0x03">Read Holding Registers (0x03)</option>
        <option value="0x04">Read Input Registers (0x04)</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        class="w-20 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Count</label>
      <input
        type="number"
        :value="config.count"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        class="w-20 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Data Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Format</label>
      <select
        :value="config.format"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
        <option value="Bin">Bin</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Byte Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Interval (ms)</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-24 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1 items-center ml-auto">
      <label class="text-xs font-medium text-gray-500">Raw</label>
      <button
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-9 h-5 rounded-full p-0.5 transition-colors relative',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-4 h-4 rounded-full bg-white shadow-sm transition-transform',
            config.raw ? 'translate-x-4' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </div>
</template>
```

---

### Task 4: Register Matrix Data Grid (`DataGrid.vue`)

**Files:**
- Create: `src/components/DataGrid.vue`

**Interfaces:**
- Props: `startAddress: number`, `count: number`, `values: Record<number, number>`
- Emits: `update-cell(address, value)`

- [ ] **Step 1: Create `src/components/DataGrid.vue`**

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';

const props = defineProps<{
  startAddress: number;
  count: number;
  values: Record<number, number>;
}>();

const emit = defineEmits<{
  (e: 'update-cell', address: number, value: number): void;
}>();

const editingAddress = ref<number | null>(null);
const editValue = ref<string>('');

const rows = computed(() => {
  const result: number[] = [];
  const startRow = Math.floor(props.startAddress / 10) * 10;
  const endRow = Math.ceil((props.startAddress + props.count) / 10) * 10;
  for (let addr = startRow; addr < endRow; addr += 10) {
    result.push(addr);
  }
  return result;
});

function startEdit(addr: number) {
  editingAddress.value = addr;
  editValue.value = (props.values[addr] ?? 0).toString();
}

function saveEdit(addr: number) {
  const num = parseInt(editValue.value, 10);
  if (!isNaN(num)) {
    emit('update-cell', addr, num);
  }
  editingAddress.value = null;
}
</script>

<template>
  <div class="flex-1 p-4 overflow-auto bg-gray-50/50">
    <div class="bg-white border border-gray-200 rounded-xl shadow-sm overflow-hidden">
      <table class="w-full text-sm text-center border-collapse">
        <thead>
          <tr class="bg-gray-50 border-b border-gray-200 text-gray-600 font-medium">
            <th class="py-3 px-4 w-28 border-r border-gray-200 font-semibold">Address</th>
            <th v-for="col in 10" :key="col - 1" class="py-3 px-2 border-r border-gray-200 last:border-r-0">
              {{ col - 1 }}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr v-for="rowAddr in rows" :key="rowAddr" class="hover:bg-blue-50/20 transition-colors">
            <td class="py-2.5 px-4 font-semibold text-gray-700 bg-gray-50/50 border-r border-gray-200 font-mono">
              {{ rowAddr }}
            </td>
            <td
              v-for="colOffset in 10"
              :key="colOffset - 1"
              @dblclick="startEdit(rowAddr + colOffset - 1)"
              class="py-2.5 px-2 border-r border-gray-200 last:border-r-0 font-mono cursor-pointer hover:bg-blue-100/50 text-gray-800 transition-colors"
            >
              <template v-if="editingAddress === (rowAddr + colOffset - 1)">
                <input
                  v-model="editValue"
                  @blur="saveEdit(rowAddr + colOffset - 1)"
                  @keyup.enter="saveEdit(rowAddr + colOffset - 1)"
                  v-focus
                  class="w-full text-center bg-white border border-blue-500 rounded px-1 py-0.5 outline-none font-mono text-sm"
                />
              </template>
              <template v-else>
                {{ props.values[rowAddr + colOffset - 1] ?? 0 }}
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
```

---

### Task 5: Status Bar & Main App Assembly (`StatusBar.vue` & `App.vue`)

**Files:**
- Create: `src/components/StatusBar.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Create `src/components/StatusBar.vue`**

```vue
<script setup lang="ts">
defineProps<{
  message: string;
  type: 'error' | 'success' | 'info';
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-2 bg-white border-t border-gray-200 text-xs select-none">
    <div class="flex items-center gap-2">
      <span
        :class="[
          'w-2 h-2 rounded-full',
          type === 'error' ? 'bg-red-500 animate-pulse' : type === 'success' ? 'bg-emerald-500' : 'bg-blue-500'
        ]"
      ></span>
      <span
        :class="[
          'font-medium',
          type === 'error' ? 'text-red-600' : type === 'success' ? 'text-emerald-700' : 'text-gray-600'
        ]"
      >
        {{ message }}
      </span>
    </div>
    <div class="text-gray-400 font-mono">
      © Modlab • dote27@163.com
    </div>
  </div>
</template>
```

- [ ] **Step 2: Assemble in `src/App.vue`**

Replace `src/App.vue` with the unified layout integrating `TabBar`, `Toolbar`, `ConfigRow`, `DataGrid`, and `StatusBar`.
