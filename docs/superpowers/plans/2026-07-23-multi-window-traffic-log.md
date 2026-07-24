# Tauri Multi-Window Traffic Log Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Tauri 2.0 native multi-window communication packet logger with device name uniqueness validation, cross-window event broadcasting, device filtering, and TX/RX direction filtering.

**Architecture:** 
- `NewDeviceModal.vue` & `TabBar.vue`: Enforce device title uniqueness across open session tabs.
- `windowService.ts`: Native Tauri `WebviewWindow` helper for creating/focusing the standalone `traffic-log-window` (`/#/traffic-log`).
- `TrafficLogWindow.vue`: Standalone Vue route view component listening to Tauri global events (`traffic-log-entry`) with device filter, TX/RX tabs, and auto-scroll.
- `App.vue`: Global packet event emitter using `@tauri-apps/api/event`.

**Tech Stack:** Vue 3 (`<script setup lang="ts">`), TypeScript, Vite, TailwindCSS, Tauri 2.0 `@tauri-apps/api/webviewWindow` and `@tauri-apps/api/event`.

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Window size: Main window 1000x700, Traffic window 850x550.
- Native multi-window (no HTML modal dialogs for traffic log).
- Unique device name validation strictly enforced.
- Zero build errors (`pnpm build`).

---

### Task 1: Device Name Uniqueness Validation (`NewDeviceModal.vue` & `TabBar.vue` & `App.vue`)

**Files:**
- Modify: `src/components/NewDeviceModal.vue`
- Modify: `src/components/TabBar.vue`
- Modify: `src/App.vue`

**Interfaces:**
- `NewDeviceModal.vue`: Props `existingNames: string[]`
- Validates name input and displays red error text if input matches any existing session name.

- [ ] **Step 1: Update `NewDeviceModal.vue`**

```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { Plus, X, Server, Cpu, AlertCircle } from '@lucide/vue';

const props = defineProps<{
  show: boolean;
  existingNames: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', deviceName: string, role: 'Slave' | 'Master'): void;
}>();

const deviceName = ref<string>('Slave 2');
const role = ref<'Slave' | 'Master'>('Slave');
const errorMessage = ref<string>('');

const isDuplicate = computed(() => {
  const name = deviceName.value.trim();
  return props.existingNames.some(n => n.toLowerCase() === name.toLowerCase());
});

function handleCreate() {
  const name = deviceName.value.trim() || (role.value === 'Slave' ? 'Slave 2' : 'Master 1');
  if (isDuplicate.value) {
    errorMessage.value = `Device name "${name}" already exists. Please enter a unique name.`;
    return;
  }
  errorMessage.value = '';
  emit('create', name, role.value);
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden animate-in fade-in zoom-in-95 duration-150 select-none">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <Plus class="w-5 h-5 text-blue-600" />
          <h3 class="font-semibold text-gray-900 text-base">Add New Device</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 space-y-4 text-sm">
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Device Name</label>
          <input
            v-model="deviceName"
            @input="errorMessage = ''"
            type="text"
            placeholder="e.g. Slave 2"
            class="w-full px-3.5 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-medium text-sm"
          />
          <div v-if="errorMessage" class="flex items-center gap-1 mt-1.5 text-xs text-red-600">
            <AlertCircle class="w-3.5 h-3.5 shrink-0" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Device Type (Role)</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="role = 'Slave'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Server class="w-4 h-4" /> Slave
            </button>
            <button
              type="button"
              @click="role = 'Master'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cpu class="w-4 h-4" /> Master
            </button>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="handleCreate" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Create Device</button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Update `TabBar.vue` for Renaming Validation**

Check for duplicate name when saving rename in `TabBar.vue`.

- [ ] **Step 3: Run `pnpm build` to verify compilation**

Run: `pnpm build`
Expected: PASS cleanly.

---

### Task 2: Tauri Multi-Window Service & Route View Setup (`windowService.ts` & `App.vue`)

**Files:**
- Create: `src/services/windowService.ts`
- Modify: `src/App.vue`

**Interfaces:**
- `openTrafficLogWindow(): Promise<void>`
- Opens or focuses native `traffic-log-window` with URL `/#/traffic-log`.

- [ ] **Step 1: Create `src/services/windowService.ts`**

```typescript
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export async function openTrafficLogWindow() {
  const existing = await WebviewWindow.getByLabel('traffic-log-window');
  if (existing) {
    await existing.focus();
    return;
  }

  const webview = new WebviewWindow('traffic-log-window', {
    url: '/#/traffic-log',
    title: 'Modbus Communication Traffic Log - Modlab',
    width: 850,
    height: 550,
    resizable: true,
    alwaysOnTop: false,
  });

  webview.once('tauri://created', function () {
    console.log('Traffic Log Window created successfully');
  });

  webview.once('tauri://error', function (e) {
    console.error('Error creating Traffic Log Window:', e);
  });
}
```

- [ ] **Step 2: Setup Simple Hash Routing in `src/App.vue`**

Support rendering `TrafficLogWindow` when `window.location.hash === '#/traffic-log'`.

---

### Task 3: Standalone Traffic Log Window Component (`TrafficLogWindow.vue`)

**Files:**
- Create: `src/components/TrafficLogWindow.vue`

**Interfaces:**
- Listens to Tauri global event `traffic-log-entry`.
- Device selector, TX/RX direction tabs, clear, always-on-top toggle, auto-scroll.

- [ ] **Step 1: Create `src/components/TrafficLogWindow.vue`**

```vue
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Trash2, Pin, ArrowDownCircle, Copy, ArrowUpRight, ArrowDownLeft } from '@lucide/vue';

export interface TrafficLogEntry {
  id: string;
  sessionId: string;
  sessionTitle: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
  timestamp: string;
}

const logs = ref<TrafficLogEntry[]>([]);
const selectedDevice = ref<string>('ALL');
const directionFilter = ref<'ALL' | 'TX' | 'RX'>('ALL');
const isAlwaysOnTop = ref<boolean>(false);
const autoScroll = ref<boolean>(true);
const containerRef = ref<HTMLDivElement | null>(null);

let unlistenFn: UnlistenFn | null = null;

const availableDevices = computed(() => {
  const set = new Set<string>();
  logs.value.forEach(log => set.add(log.sessionTitle));
  return Array.from(set);
});

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    const matchDevice = selectedDevice.value === 'ALL' || log.sessionTitle === selectedDevice.value;
    const matchDir = directionFilter.value === 'ALL' || log.direction === directionFilter.value;
    return matchDevice && matchDir;
  });
});

async function toggleAlwaysOnTop() {
  try {
    const appWindow = getCurrentWebviewWindow();
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await appWindow.setAlwaysOnTop(isAlwaysOnTop.value);
  } catch (err) {
    console.error('Failed to set always on top:', err);
  }
}

function clearLogs() {
  logs.value = [];
}

function copyBytes(bytes: string) {
  navigator.clipboard.writeText(bytes);
}

onMounted(async () => {
  try {
    unlistenFn = await listen<TrafficLogEntry>('traffic-log-entry', (event) => {
      logs.value.unshift(event.payload);
      if (logs.value.length > 500) {
        logs.value.pop();
      }
      if (autoScroll.value) {
        nextTick(() => {
          if (containerRef.value) {
            containerRef.value.scrollTop = 0;
          }
        });
      }
    });
  } catch (err) {
    console.error('Failed to register traffic event listener:', err);
  }
});

onUnmounted(() => {
  if (unlistenFn) {
    unlistenFn();
  }
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-slate-900 text-slate-100 font-sans select-none overflow-hidden">
    <!-- Top Control Bar -->
    <div class="flex items-center justify-between px-4 py-3 bg-slate-800/90 border-b border-slate-700 shadow-md shrink-0">
      <div class="flex items-center gap-3">
        <!-- Device Filter -->
        <div class="flex items-center gap-2">
          <label class="text-xs font-semibold text-slate-400">Device:</label>
          <select
            v-model="selectedDevice"
            class="bg-slate-700 text-slate-200 border border-slate-600 rounded-lg px-3 py-1 text-xs outline-none focus:border-blue-500 font-medium cursor-pointer"
          >
            <option value="ALL">All Devices</option>
            <option v-for="dev in availableDevices" :key="dev" :value="dev">{{ dev }}</option>
          </select>
        </div>

        <!-- Direction Filter Tabs -->
        <div class="flex bg-slate-700/80 p-0.5 rounded-lg text-xs font-medium border border-slate-600">
          <button
            @click="directionFilter = 'ALL'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'ALL' ? 'bg-blue-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            ALL
          </button>
          <button
            @click="directionFilter = 'TX'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'TX' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            TX Only
          </button>
          <button
            @click="directionFilter = 'RX'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'RX' ? 'bg-emerald-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            RX Only
          </button>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2">
        <button
          @click="toggleAlwaysOnTop"
          :class="['p-1.5 rounded-lg border text-xs flex items-center gap-1 font-medium transition cursor-pointer', isAlwaysOnTop ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-700/60 text-slate-300 border-slate-600 hover:bg-slate-700']"
          title="Toggle Always on Top"
        >
          <Pin class="w-3.5 h-3.5" /> {{ isAlwaysOnTop ? 'Pinned' : 'Pin' }}
        </button>

        <button
          @click="autoScroll = !autoScroll"
          :class="['p-1.5 rounded-lg border text-xs flex items-center gap-1 font-medium transition cursor-pointer', autoScroll ? 'bg-blue-500/20 text-blue-300 border-blue-500/40' : 'bg-slate-700/60 text-slate-300 border-slate-600 hover:bg-slate-700']"
          title="Toggle Auto Scroll"
        >
          <ArrowDownCircle class="w-3.5 h-3.5" /> Auto Scroll
        </button>

        <button
          @click="clearLogs"
          class="p-1.5 rounded-lg bg-red-500/20 text-red-300 border border-red-500/40 hover:bg-red-500/30 text-xs flex items-center gap-1 font-medium transition cursor-pointer"
          title="Clear Traffic Logs"
        >
          <Trash2 class="w-3.5 h-3.5" /> Clear
        </button>
      </div>
    </div>

    <!-- Traffic Log List View -->
    <div ref="containerRef" class="flex-1 overflow-y-auto p-3 space-y-2 font-mono text-xs">
      <div v-if="filteredLogs.length === 0" class="flex flex-col items-center justify-center h-full text-slate-500">
        <p>No traffic logs captured.</p>
      </div>

      <div
        v-for="entry in filteredLogs"
        :key="entry.id"
        class="bg-slate-800/60 border border-slate-700/80 rounded-xl p-2.5 flex items-center justify-between gap-3 hover:border-slate-600 transition group"
      >
        <div class="flex items-center gap-3 overflow-hidden">
          <span class="text-slate-400 text-[11px] shrink-0">{{ entry.timestamp }}</span>
          
          <span class="px-2 py-0.5 rounded bg-slate-700 text-slate-200 font-semibold text-[11px] shrink-0 border border-slate-600">
            {{ entry.sessionTitle }}
          </span>

          <span
            :class="[
              'flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-bold shrink-0',
              entry.direction === 'TX' ? 'bg-sky-500/20 text-sky-400 border border-sky-500/30' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
            ]"
          >
            <component :is="entry.direction === 'TX' ? ArrowUpRight : ArrowDownLeft" class="w-3 h-3" />
            {{ entry.direction }}
          </span>

          <span class="text-slate-300 font-medium shrink-0">{{ entry.message }}</span>

          <span class="text-slate-400 truncate tracking-wide text-slate-300 font-mono">{{ entry.bytes }}</span>
        </div>

        <button
          @click="copyBytes(entry.bytes)"
          class="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-slate-200 hover:bg-slate-700 rounded transition cursor-pointer shrink-0"
          title="Copy Bytes"
        >
          <Copy class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>
```

---

### Task 4: Multi-Window Launcher Buttons & End-to-End Build Verification

**Files:**
- Modify: `src/components/Toolbar.vue`
- Modify: `src/components/StatusBar.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Add `Traffic Log` Launcher Button to `Toolbar.vue`**

Add button with `Activity` icon triggering `openTrafficLogWindow()`.

- [ ] **Step 2: Emit `traffic-log-entry` Events in `App.vue`**

Import `emit` from `@tauri-apps/api/event` and emit `traffic-log-entry` when TX/RX frames are created during session polling.

- [ ] **Step 3: Run Full Build Verification**

Run: `pnpm build`
Expected: PASS cleanly with 0 TypeScript / Vue compile errors.

- [ ] **Step 4: Commit All Changes**

```bash
git add .
git commit -m "feat(multiwindow): implement Tauri native multi-window traffic log with device and TX/RX filters"
```
