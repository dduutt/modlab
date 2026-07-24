# Industrial Modbus Enhancements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Float32/Int32 32-bit 2-register data decoding with 4-byte endianness swapping (`ABCD`, `CDAB`, `BADC`, `DCBA`), CSV point table import/export, and connection timeout & retries settings.

**Architecture:** 
- `modbusFormatter.ts`: Math utilities for Float32/Int32 IEEE 754 conversion and 4-byte swapping (`ABCD`, `CDAB`, `BADC`, `DCBA`).
- `DataGrid.vue`: 2-register cell merging when `dataType` is `Float32` / `Int32` / `UInt32` + 32-bit inline editing.
- `csvHandler.ts`: CSV generator (`Address,Value`) and CSV reader for importing/exporting point maps.
- `SettingsModal.vue` & `types.rs`: Add `timeoutMs` and `retries` fields to connection config and Rust backend structs.

**Tech Stack:** Vue 3 (`<script setup lang="ts">`), TypeScript, Vite, TailwindCSS, Rust (Tauri v2).

## Global Constraints

- Target OS: Windows (Tauri desktop app).
- Window size: 1000x700.
- CSV format only for point tables (no JSON/private formats).
- Zero build errors (`pnpm build`) and zero Rust warnings (`cargo check`).

---

### Task 1: 32-bit Float32 / Int32 & Endianness Engine (`src/utils/modbusFormatter.ts`)

**Files:**
- Modify: `src/utils/modbusFormatter.ts`

**Interfaces:**
- `formatRegisterValue(raw: number, format: string, dataType?: string, byteOrder?: string, secondRaw?: number): string`
- `parseFormattedRegisterValue(input: string, format: string, dataType?: string, byteOrder?: string): { high: number; low: number } | number`

- [ ] **Step 1: Update `src/utils/modbusFormatter.ts`**

```typescript
// Swaps 4 bytes based on Modbus byte order
function swapBytes32(bytes: Uint8Array, byteOrder: string = 'ABCD'): Uint8Array {
  const [a, b, c, d] = bytes;
  switch (byteOrder) {
    case 'CDAB': return new Uint8Array([c, d, a, b]); // Little-Endian Byte Swap
    case 'BADC': return new Uint8Array([b, a, d, c]); // Word Swap
    case 'DCBA': return new Uint8Array([d, c, b, a]); // Big-Endian Byte Swap
    case 'ABCD':
    default: return new Uint8Array([a, b, c, d]);     // Big-Endian
  }
}

function unswapBytes32(bytes: Uint8Array, byteOrder: string = 'ABCD'): Uint8Array {
  const [a, b, c, d] = bytes;
  switch (byteOrder) {
    case 'CDAB': return new Uint8Array([c, d, a, b]);
    case 'BADC': return new Uint8Array([b, a, d, c]);
    case 'DCBA': return new Uint8Array([d, c, b, a]);
    case 'ABCD':
    default: return new Uint8Array([a, b, c, d]);
  }
}

export function formatRegisterValue(
  raw: number,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  secondRaw?: number
): string {
  const low = raw ?? 0;
  
  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const high = secondRaw ?? 0;
    // Combine 2 16-bit words into 4 bytes (Big-Endian initial placement)
    const rawBytes = new Uint8Array([
      (low >> 8) & 0xff,
      low & 0xff,
      (high >> 8) & 0xff,
      high & 0xff,
    ]);

    const swapped = swapBytes32(rawBytes, byteOrder);
    const view = new DataView(swapped.buffer);

    if (dataType === 'Float32') {
      const val = view.getFloat32(0, false);
      return Number.isNaN(val) ? 'NaN' : val.toFixed(2);
    }
    if (dataType === 'Int32') {
      return view.getInt32(0, false).toString();
    }
    if (dataType === 'UInt32') {
      return view.getUint32(0, false).toString();
    }
  }

  if (format === 'Hex') {
    return '0x' + (low & 0xffff).toString(16).toUpperCase().padStart(4, '0');
  }
  return low.toString();
}

export function parseFormattedRegisterValue(
  input: string,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD'
): { word1: number; word2: number } | number {
  const trimmed = input.trim();

  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);

    if (dataType === 'Float32') {
      const floatVal = parseFloat(trimmed) || 0;
      view.setFloat32(0, floatVal, false);
    } else if (dataType === 'Int32') {
      const intVal = parseInt(trimmed, 10) || 0;
      view.setInt32(0, intVal, false);
    } else {
      const uintVal = parseInt(trimmed, 10) || 0;
      view.setUint32(0, uintVal, false);
    }

    const bytes = new Uint8Array(buffer);
    const unswapped = unswapBytes32(bytes, byteOrder);
    const word1 = (unswapped[0] << 8) | unswapped[1];
    const word2 = (unswapped[2] << 8) | unswapped[3];

    return { word1, word2 };
  }

  if (format === 'Hex' || trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return parseInt(trimmed.replace(/^0x/i, ''), 16) || 0;
  }
  return parseInt(trimmed, 10) || 0;
}
```

- [ ] **Step 2: Run `pnpm build` to verify compilation**

Run: `pnpm build`
Expected: PASS cleanly.

---

### Task 2: DataGrid 2-Register Cell Merging & 32-bit Editing (`DataGrid.vue`)

**Files:**
- Modify: `src/components/DataGrid.vue`

**Interfaces:**
- Consumes: `dataType` and `byteOrder` props.
- Displays 32-bit merged values for `Float32`, `Int32`, `UInt32` data types and writes back to both registers.

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
  dataType?: string;
  byteOrder?: string;
  functionCode?: string;
}>();

const emit = defineEmits<{
  (e: 'update-cell', address: number, value: number): void;
  (e: 'update-cell-pair', address1: number, value1: number, address2: number, value2: number): void;
}>();

const editingAddress = ref<number | null>(null);
const editValue = ref<string>('');
const inputRef = ref<HTMLInputElement | null>(null);

const contextMenu = ref<{
  show: boolean;
  x: number;
  y: number;
  address: number | null;
}>({ show: false, x: 0, y: 0, address: null });

const isCoil = computed(() => {
  return props.functionCode === '0x01' || props.functionCode === '0x02';
});

const is32Bit = computed(() => {
  return props.dataType === 'Float32' || props.dataType === 'Int32' || props.dataType === 'UInt32';
});

const activeFormat = computed(() => (isCoil.value ? 'Dec' : props.format || 'Dec'));

const step = computed(() => (is32Bit.value ? 2 : 1));

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
    const current = props.values[addr] ?? 0;
    emit('update-cell', addr, current === 1 ? 0 : 1);
  } else {
    startEdit(addr);
  }
}

async function startEdit(addr: number) {
  editingAddress.value = addr;
  const raw1 = props.values[addr] ?? 0;
  const raw2 = props.values[addr + 1] ?? 0;
  editValue.value = formatRegisterValue(raw1, activeFormat.value, props.dataType, props.byteOrder, raw2);
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
}

function saveEdit(addr: number) {
  const parsed = parseFormattedRegisterValue(editValue.value, activeFormat.value, props.dataType, props.byteOrder);
  if (typeof parsed === 'object') {
    emit('update-cell-pair', addr, parsed.word1, addr + 1, parsed.word2);
  } else if (!isNaN(parsed)) {
    emit('update-cell', addr, parsed);
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
    const addr = contextMenu.value.address;
    const raw1 = props.values[addr] ?? 0;
    const raw2 = props.values[addr + 1] ?? 0;
    const text = formatRegisterValue(raw1, activeFormat.value, props.dataType, props.byteOrder, raw2);
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
    const addr = contextMenu.value.address;
    if (is32Bit.value) {
      emit('update-cell-pair', addr, 0, addr + 1, 0);
    } else {
      emit('update-cell', addr, 0);
    }
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
                    'inline-block px-2.5 py-0.5 rounded-full text-xs font-semibold font-mono transition select-none',
                    (props.values[rowAddr + colOffset - 1] ?? 0) === 1
                      ? 'bg-emerald-100 text-emerald-700 border border-emerald-300'
                      : 'bg-gray-100 text-gray-500 border border-gray-200'
                  ]"
                >
                  {{ (props.values[rowAddr + colOffset - 1] ?? 0) === 1 ? '1' : '0' }}
                </span>
              </template>
              <template v-else>
                {{ formatRegisterValue(props.values[rowAddr + colOffset - 1] ?? 0, activeFormat, props.dataType, props.byteOrder, props.values[(rowAddr + colOffset - 1) + 1]) }}
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
      <button @click="copyValue" class="flex items-center gap-2 px-3 py-1.5 hover:bg-gray-100 w-full text-left font-medium cursor-pointer">
        <Copy class="w-3.5 h-3.5 text-gray-500" /> Copy Value
      </button>
      <button @click="triggerEdit" class="flex items-center gap-2 px-3 py-1.5 hover:bg-gray-100 w-full text-left font-medium cursor-pointer">
        <Edit3 class="w-3.5 h-3.5 text-gray-500" /> Edit Value
      </button>
      <div class="my-1 border-t border-gray-100"></div>
      <button @click="resetToZero" class="flex items-center gap-2 px-3 py-1.5 hover:bg-red-50 text-red-600 w-full text-left font-medium cursor-pointer">
        <RotateCcw class="w-3.5 h-3.5 text-red-500" /> Reset to 0
      </button>
    </div>
  </div>
</template>
```

---

### Task 3: CSV Import & Export Utilities (`src/utils/csvHandler.ts` & `Toolbar.vue` / `App.vue`)

**Files:**
- Create: `src/utils/csvHandler.ts`
- Modify: `src/components/Toolbar.vue`
- Modify: `src/App.vue`

**Interfaces:**
- `exportToCSV(sessionName: string, values: Record<number, number>): void`
- `parseCSVFile(file: File): Promise<Record<number, number>>`

- [ ] **Step 1: Create `src/utils/csvHandler.ts`**

```typescript
export function exportToCSV(sessionName: string, values: Record<number, number>) {
  let csvContent = 'Address,Value\n';
  const addresses = Object.keys(values).map(Number).sort((a, b) => a - b);
  for (const addr of addresses) {
    csvContent += `${addr},${values[addr]}\n`;
  }

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${sessionName.replace(/\s+/g, '_')}_registers.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function parseCSVFile(file: File): Promise<Record<number, number>> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const text = e.target?.result as string;
      if (!text) {
        return resolve({});
      }
      const lines = text.split(/\r?\n/);
      const result: Record<number, number> = {};
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        if (!line || line.startsWith('Address')) continue; // Skip header
        const [addrStr, valStr] = line.split(',');
        const addr = parseInt(addrStr?.trim(), 10);
        const val = parseInt(valStr?.trim(), 10);
        if (!isNaN(addr) && !isNaN(val)) {
          result[addr] = val;
        }
      }
      resolve(result);
    };
    reader.onerror = (err) => reject(err);
    reader.readAsText(file);
  });
}
```

- [ ] **Step 2: Add Import & Export Buttons to `Toolbar.vue`**

Add `Import CSV` button with hidden `<input type="file" accept=".csv">` file trigger in `Toolbar.vue`.

- [ ] **Step 3: Connect CSV Handlers in `App.vue`**

Wire `exportToCSV` and `parseCSVFile` in `src/App.vue`.

---

### Task 4: Timeout & Retries Parameters in Settings Modal (`SettingsModal.vue` & `types.rs`)

**Files:**
- Modify: `src/components/SettingsModal.vue`
- Modify: `src-tauri/src/modbus/types.rs`
- Modify: `src/App.vue`

**Interfaces:**
- Add `timeoutMs: number` (default `1000`) and `retries: number` (default `3`) to `ConnectionConfig`.

- [ ] **Step 1: Update `SettingsModal.vue`**

Add `Timeout (ms)` and `Retries` input fields under TCP & RTU configuration blocks in `SettingsModal.vue`.

- [ ] **Step 2: Update Rust `types.rs` Struct**

Add `timeout_ms: u32` and `retries: u8` to `ConnectionConfig` in `src-tauri/src/modbus/types.rs`.

- [ ] **Step 3: Verify Builds**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Run: `pnpm build`
Expected: PASS cleanly with zero errors.

- [ ] **Step 4: Commit All Changes**

```bash
git add .
git commit -m "feat(industrial): complete Float32/Int32 endianness, CSV import/export, and timeout retries"
```
