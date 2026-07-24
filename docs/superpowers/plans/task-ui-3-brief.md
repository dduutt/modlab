# Task 3 Brief: DataGrid Function Code Dual Behavior & Context Menu (`DataGrid.vue`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS

## Instructions
Update `src/components/DataGrid.vue` to support:
1. Dual double-click logic:
   - Coils (`0x01` / `0x02`): double-clicking a cell directly flips value between `0` and `1`. Value `1` renders as a highlighted green active pill badge.
   - Registers (`0x03` / `0x04`): double-clicking opens text `<input>`.
2. Right-Click Context Menu:
   - Copy Value: copies cell text to clipboard.
   - Edit Value: triggers cell edit/flip.
   - Reset to 0: sets cell value to `0`.

Code for `src/components/DataGrid.vue`:
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

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-ui-3-report.md`.
