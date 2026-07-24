# UI & Interaction Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement comprehensive UX & interaction enhancements for the Modbus Tauri desktop application, including New Device Modal, Tab Inline Renaming, Connection Close Safety Guard, DataGrid Coil vs Register Double-Click logic, Context Menu, and ConfigRow Function-Format linkage.

**Architecture:** 
- `NewDeviceModal.vue`: Modal dialog for creating a session (device name + role).
- `TabBar.vue`: Inline title editing on double-click + close confirmation safety guard when connected.
- `DataGrid.vue`: Dual double-click handler (instant 0/1 flip for 0x01/0x02 coils vs text input for 0x03/0x04 registers) + custom right-click context menu.
- `ConfigRow.vue`: Automatic disabling of Format select when 0x01/0x02 is active.
- `Toolbar.vue`: Loading state on Connect button + disabled state on Random/Increment for read-only codes.

**Tech Stack:** Vue 3 (`<script setup lang="ts">`), TypeScript, Vite, TailwindCSS, Lucide Icons (`@lucide/vue`).

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Window size: 1000x700.
- No theme switcher (clean light theme only).
- Zero build errors (`pnpm build`).

---

### Task 1: New Device Modal Component (`NewDeviceModal.vue`)

**Files:**
- Create: `src/components/NewDeviceModal.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Props: `show: boolean`
- Emits: `close`, `create(deviceName: string, role: 'Slave' | 'Master')`

- [ ] **Step 1: Create `src/components/NewDeviceModal.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { Plus, X, Server, Cpu } from '@lucide/vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', deviceName: string, role: 'Slave' | 'Master'): void;
}>();

const deviceName = ref<string>('Slave 2');
const role = ref<'Slave' | 'Master'>('Slave');

function handleCreate() {
  const name = deviceName.value.trim() || (role.value === 'Slave' ? 'Slave 2' : 'Master 1');
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
            type="text"
            placeholder="e.g. Slave 2"
            class="w-full px-3.5 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-medium text-sm"
          />
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

- [ ] **Step 2: Connect `NewDeviceModal.vue` in `App.vue`**

Update `App.vue` to show `NewDeviceModal` when `handleAddTab` is called.

- [ ] **Step 3: Test Build**

Run: `pnpm build`
Expected: PASS

---

### Task 2: Tab Inline Renaming & Close Safety Guard Modal (`TabBar.vue` & `App.vue`)

**Files:**
- Create: `src/components/ConfirmModal.vue`
- Modify: `src/components/TabBar.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Emits: `rename-tab(id, newTitle)` in `TabBar.vue`.
- `ConfirmModal.vue`: Confirmation dialog for disconnecting active device.

- [ ] **Step 1: Create `src/components/ConfirmModal.vue`**

```vue
<script setup lang="ts">
import { AlertTriangle, X } from '@lucide/vue';

defineProps<{
  show: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'confirm'): void;
}>();
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-red-50/50">
        <div class="flex items-center gap-2 text-red-600">
          <AlertTriangle class="w-5 h-5" />
          <h3 class="font-semibold text-gray-900 text-base">{{ title }}</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 text-sm text-gray-600">
        {{ message }}
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="emit('confirm')" class="px-5 py-1.5 rounded-xl bg-red-600 hover:bg-red-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Confirm Close</button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Update `TabBar.vue` for Inline Title Renaming**

Update `TabBar.vue` to allow double-clicking tab text to rename inline.

- [ ] **Step 3: Connect Close Safety Guard in `App.vue`**

If tab is connected, prompt `ConfirmModal` before closing.

---

### Task 3: DataGrid Function Code Dual Behavior & Context Menu (`DataGrid.vue`)

**Files:**
- Modify: `src/components/DataGrid.vue`

**Interfaces:**
- Coils (0x01/0x02): Double-clicking directly flips cell value between `0` and `1`.
- Registers (0x03/0x04): Double-clicking opens text `<input>`.
- Custom Context Menu: Copy Value, Edit Value, Reset to 0.

- [ ] **Step 1: Update `DataGrid.vue`**

```vue
<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import { formatRegisterValue, parseFormattedRegisterValue } from '../utils/modbusFormatter';
import { Copy, Edit3, RotateCcw } from '@lucide/vue';

const props = defineProps<{
  startAddress: number;
  count: number;
  values: Record<number, number>;
  format?: string;
  functionCode?: string;
}>();

const emit = defineEmits<{
  (e: 'update-cell', address: number, value: number): void;
}>();

const editingAddress = ref<number | null>(null);
const editValue = ref<string>('');
const inputRef = ref<HTMLInputElement | null>(null);

// Context Menu State
const contextMenu = ref<{
  show: boolean;
  x: number;
  y: number;
  address: number | null;
}>({ show: false, x: 0, y: 0, address: null });

const isCoil = computed(() => {
  return props.functionCode === '0x01' || props.functionCode === '0x02';
});

const activeFormat = computed(() => (isCoil.value ? 'Dec' : props.format || 'Dec'));

const rows = computed(() => {
  const result: number[] = [];
  const startRow = Math.floor(props.startAddress / 10) * 10;
  const totalCount = props.count > 0 ? props.count : 40;
  const endRow = Math.ceil((props.startAddress + totalCount) / 10) * 10;
  for (let addr = startRow; addr < endRow; addr += 10) {
    result.push(addr);
  }
  return result;
});

function handleCellDoubleClick(addr: number) {
  if (isCoil.value) {
    // Instant flip 0 <-> 1 for coils/discrete inputs
    const current = props.values[addr] ?? 0;
    emit('update-cell', addr, current === 1 ? 0 : 1);
  } else {
    // Open text input for registers
    startEdit(addr);
  }
}

async function startEdit(addr: number) {
  editingAddress.value = addr;
  const raw = props.values[addr] ?? 0;
  editValue.value = formatRegisterValue(raw, activeFormat.value);
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
}

function saveEdit(addr: number) {
  const num = parseFormattedRegisterValue(editValue.value, activeFormat.value);
  if (!isNaN(num)) {
    emit('update-cell', addr, num);
  }
  editingAddress.value = null;
}

function handleContextMenu(e: MouseEvent, addr: number) {
  e.preventDefault();
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    address: addr,
  };
}

function closeContextMenu() {
  contextMenu.value.show = false;
}

function copyValue() {
  if (contextMenu.value.address !== null) {
    const val = props.values[contextMenu.value.address] ?? 0;
    const text = formatRegisterValue(val, activeFormat.value);
    navigator.clipboard.writeText(text);
  }
  closeContextMenu();
}

function triggerEdit() {
  if (contextMenu.value.address !== null) {
    handleCellDoubleClick(contextMenu.value.address);
  }
  closeContextMenu();
}

function resetToZero() {
  if (contextMenu.value.address !== null) {
    emit('update-cell', contextMenu.value.address, 0);
  }
  closeContextMenu();
}
</script>

<template>
  <div class="flex-1 p-4 overflow-auto bg-gray-50/50 relative" @click="closeContextMenu">
    <div class="bg-white border border-gray-200 rounded-xl shadow-2xs overflow-hidden">
      <table class="w-full text-sm text-center border-collapse">
        <thead>
          <tr class="bg-gray-50/80 border-b border-gray-200 text-gray-600 font-medium">
            <th class="py-3 px-4 w-28 border-r border-gray-200 font-semibold select-none">Address</th>
            <th v-for="col in 10" :key="col - 1" class="py-3 px-2 border-r border-gray-200 last:border-r-0 select-none">
              {{ col - 1 }}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr v-for="rowAddr in rows" :key="rowAddr" class="hover:bg-blue-50/20 transition-colors">
            <td class="py-2.5 px-4 font-semibold text-gray-700 bg-gray-50/50 border-r border-gray-200 font-mono select-none">
              {{ rowAddr }}
            </td>
            <td
              v-for="colOffset in 10"
              :key="colOffset - 1"
              @dblclick="handleCellDoubleClick(rowAddr + colOffset - 1)"
              @contextmenu="handleContextMenu($event, rowAddr + colOffset - 1)"
              class="py-2.5 px-2 border-r border-gray-200 last:border-r-0 font-mono cursor-pointer hover:bg-blue-100/40 text-gray-800 transition-colors select-none"
            >
              <template v-if="editingAddress === (rowAddr + colOffset - 1)">
                <input
                  ref="inputRef"
                  v-model="editValue"
                  @blur="saveEdit(rowAddr + colOffset - 1)"
                  @keyup.enter="saveEdit(rowAddr + colOffset - 1)"
                  class="w-full text-center bg-white border border-blue-500 rounded px-1 py-0.5 outline-none font-mono text-sm shadow-xs"
                />
              </template>
              <template v-else-if="isCoil">
                <span
                  :class="[
                    'inline-block px-2.5 py-0.5 rounded-full text-xs font-semibold font-mono transition',
                    (props.values[rowAddr + colOffset - 1] ?? 0) === 1
                      ? 'bg-emerald-100 text-emerald-700 border border-emerald-300'
                      : 'bg-gray-100 text-gray-500 border border-gray-200'
                  ]"
                >
                  {{ (props.values[rowAddr + colOffset - 1] ?? 0) === 1 ? '1' : '0' }}
                </span>
              </template>
              <template v-else>
                {{ formatRegisterValue(props.values[rowAddr + colOffset - 1] ?? 0, activeFormat) }}
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Context Menu -->
    <div
      v-if="contextMenu.show"
      :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
      class="fixed z-50 bg-white border border-gray-200 shadow-xl rounded-xl py-1 w-36 text-xs text-gray-700 animate-in fade-in zoom-in-95 duration-100 select-none"
    >
      <button @click="copyValue" class="flex items-center gap-2 px-3 py-1.5 hover:bg-gray-100 w-full text-left font-medium">
        <Copy class="w-3.5 h-3.5 text-gray-500" /> Copy Value
      </button>
      <button @click="triggerEdit" class="flex items-center gap-2 px-3 py-1.5 hover:bg-gray-100 w-full text-left font-medium">
        <Edit3 class="w-3.5 h-3.5 text-gray-500" /> Edit Value
      </button>
      <div class="my-1 border-t border-gray-100"></div>
      <button @click="resetToZero" class="flex items-center gap-2 px-3 py-1.5 hover:bg-red-50 text-red-600 w-full text-left font-medium">
        <RotateCcw class="w-3.5 h-3.5 text-red-500" /> Reset to 0
      </button>
    </div>
  </div>
</template>
```

---

### Task 4: ConfigRow Function-Format Linkage & Toolbar State (`ConfigRow.vue` & `Toolbar.vue`)

**Files:**
- Modify: `src/components/ConfigRow.vue`
- Modify: `src/components/Toolbar.vue`

**Interfaces:**
- Format select disabled when `functionCode === '0x01'` or `'0x02'`.
- `Toolbar.vue` Random/Increment buttons disabled when functionCode is read-only (`0x02` / `0x04`).

- [ ] **Step 1: Update `ConfigRow.vue` Linkage**

Disable `Format` select when `functionCode` is `0x01` or `0x02`.

- [ ] **Step 2: Update `Toolbar.vue` Feedback**

Disable `Random` and `Increment` when function code is read-only.

---

### Task 5: Status Bar Error Display & End-to-End Build Verification

**Files:**
- Modify: `src/components/StatusBar.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Update `StatusBar.vue` Concise Display**

Display concise red error text directly on bottom-left.

- [ ] **Step 2: Run Full Build Verification**

Run: `pnpm build`
Expected: PASS cleanly with 0 TypeScript / Vue compile errors.

- [ ] **Step 3: Commit All Changes**

```bash
git add .
git commit -m "feat(ui): complete UI and interaction refactoring according to design spec"
```
