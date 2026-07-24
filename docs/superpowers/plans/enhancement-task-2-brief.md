# Enhancement Task 2 Brief: Data Formatter & DataGrid Format Integration

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)

## Instructions
1. Create `src/utils/modbusFormatter.ts` to convert numbers between Dec, Hex (e.g. `0x000A`), and Bin (e.g. `0000 0000 0000 1010`), and parse formatted strings back into numbers.
2. Update `src/components/DataGrid.vue` to accept `format: string` ('Dec' | 'Hex' | 'Bin') and `dataType: string` props, formatting cell values dynamically.

Code for `src/utils/modbusFormatter.ts`:
```typescript
export function formatRegisterValue(raw: number, format: string): string {
  const num = raw ?? 0;
  if (format === 'Hex') {
    return '0x' + (num & 0xffff).toString(16).toUpperCase().padStart(4, '0');
  }
  if (format === 'Bin') {
    const binStr = (num & 0xffff).toString(2).padStart(16, '0');
    return binStr.replace(/(.{4})/g, '$1 ').trim();
  }
  return num.toString();
}

export function parseFormattedRegisterValue(input: string, format: string): number {
  const trimmed = input.trim();
  if (format === 'Hex' || trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return parseInt(trimmed.replace(/^0x/i, ''), 16) || 0;
  }
  if (format === 'Bin') {
    return parseInt(trimmed.replace(/\s+/g, ''), 2) || 0;
  }
  return parseInt(trimmed, 10) || 0;
}
```

Code for `src/components/DataGrid.vue`:
```vue
<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import { formatRegisterValue, parseFormattedRegisterValue } from '../utils/modbusFormatter';

const props = defineProps<{
  startAddress: number;
  count: number;
  values: Record<number, number>;
  format?: string;
  dataType?: string;
}>();

const emit = defineEmits<{
  (e: 'update-cell', address: number, value: number): void;
}>();

const editingAddress = ref<number | null>(null);
const editValue = ref<string>('');
const inputRef = ref<HTMLInputElement | null>(null);

const activeFormat = computed(() => props.format || 'Dec');

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
</script>

<template>
  <div class="flex-1 p-4 overflow-auto bg-gray-50/50">
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
              @dblclick="startEdit(rowAddr + colOffset - 1)"
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
              <template v-else>
                {{ formatRegisterValue(props.values[rowAddr + colOffset - 1] ?? 0, activeFormat) }}
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/enhancement-task-2-report.md`.
