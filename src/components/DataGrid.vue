<script setup lang="ts">
import { computed, ref } from 'vue';
import { formatRegisterValue } from '../utils/modbusFormatter';
import { Copy, Edit3, RotateCcw } from '@lucide/vue';
import EditValueModal from './EditValueModal.vue';

const props = defineProps<{
  startAddress: number;
  count: number;
  values: Record<number, number>;
  format?: string;
  dataType?: string;
  byteOrder?: string;
  functionCode?: string;
  raw?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update-cell', address: number, value: number): void;
  (e: 'update-cell-pair', address1: number, value1: number, address2: number, value2: number): void;
}>();

function getRawHex(addr: number): string {
  const val = (props.values[addr] ?? 0) & 0xffff;
  return '0x' + val.toString(16).toUpperCase().padStart(4, '0');
}

const showEditModal = ref<boolean>(false);
const modalAddress = ref<number | null>(null);
const modalCurrentValueText = ref<string>('');

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

function getPrimaryCellText(addr: number): string {
  if (isCoil.value) {
    return (props.values[addr] ?? 0) === 1 ? '1' : '0';
  }
  if (is32Bit.value) {
    const offset = addr - props.startAddress;
    if (offset % 2 !== 0) {
      return '-';
    }
    const raw1 = props.values[addr] ?? 0;
    const raw2 = props.values[addr + 1] ?? 0;
    return formatRegisterValue(raw1, activeFormat.value, props.dataType, props.byteOrder, raw2);
  }
  return formatRegisterValue(props.values[addr] ?? 0, activeFormat.value, props.dataType, props.byteOrder);
}

function handleCellDoubleClick(addr: number) {
  if (isCoil.value) {
    const current = props.values[addr] ?? 0;
    emit('update-cell', addr, current === 1 ? 0 : 1);
  } else {
    let targetAddr = addr;
    if (is32Bit.value && (addr - props.startAddress) % 2 !== 0) {
      targetAddr = addr - 1;
    }
    modalAddress.value = targetAddr;
    modalCurrentValueText.value = getPrimaryCellText(targetAddr);
    showEditModal.value = true;
  }
}

function handleModalSave(addr: number, parsed: { word1: number; word2?: number } | number) {
  if (typeof parsed === 'object') {
    emit('update-cell-pair', addr, parsed.word1, addr + 1, parsed.word2 ?? 0);
  } else if (!isNaN(parsed)) {
    emit('update-cell', addr, parsed);
  }
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
    let addr = contextMenu.value.address;
    if (is32Bit.value && (addr - props.startAddress) % 2 !== 0) {
      addr = addr - 1;
    }
    const text = getPrimaryCellText(addr);
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
    let addr = contextMenu.value.address;
    if (is32Bit.value && (addr - props.startAddress) % 2 !== 0) {
      addr = addr - 1;
    }
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
            <th class="py-2.5 px-2 w-20 shrink-0 border-r border-gray-200 font-semibold select-none">Address</th>
            <th v-for="col in 10" :key="col - 1" class="py-2.5 px-2 min-w-[70px] whitespace-nowrap border-r border-gray-200 last:border-r-0 select-none">
              {{ col - 1 }}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr
            v-for="(rowAddr, index) in rows"
            :key="rowAddr"
            :class="[
              'transition-colors hover:bg-blue-50/40',
              index % 2 === 1 ? 'bg-slate-50/60' : 'bg-white'
            ]"
          >
            <td
              :class="[
                'py-2.5 px-2 font-semibold text-gray-700 border-r border-gray-200 font-mono select-none',
                index % 2 === 1 ? 'bg-slate-100/70' : 'bg-gray-50/80'
              ]"
            >
              {{ rowAddr }}
            </td>
            <td
              v-for="colOffset in 10"
              :key="colOffset - 1"
              @dblclick="handleCellDoubleClick(rowAddr + colOffset - 1)"
              @contextmenu="handleContextMenu($event, rowAddr + colOffset - 1)"
              class="py-2.5 px-2 min-w-[70px] whitespace-nowrap border-r border-gray-200 last:border-r-0 font-mono cursor-pointer hover:bg-blue-100/40 text-gray-800 transition-colors select-none"
            >
              <div class="flex flex-col items-center justify-center py-0.5 leading-tight">
                <span>{{ getPrimaryCellText(rowAddr + colOffset - 1) }}</span>
                <span v-if="props.raw" class="text-[10px] text-gray-400 font-mono tracking-wider font-normal leading-none mt-0.5 select-none">
                  {{ getRawHex(rowAddr + colOffset - 1) }}
                </span>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Edit Value Popup Modal -->
    <EditValueModal
      :show="showEditModal"
      :address="modalAddress"
      :currentValueText="modalCurrentValueText"
      :format="activeFormat"
      :dataType="props.dataType || 'Int16'"
      :byteOrder="props.byteOrder || 'ABCD'"
      :is32Bit="is32Bit"
      @close="showEditModal = false"
      @save="handleModalSave"
    />

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
