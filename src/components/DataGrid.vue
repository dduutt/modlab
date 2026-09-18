<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { formatRegisterValue } from '../utils/modbusFormatter';
import { Copy, Edit3, RotateCcw, ToggleLeft } from '@lucide/vue';
import EditValueModal from './EditValueModal.vue';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

const props = defineProps<{
  startAddress: number;
  count: number;
  values: Record<number, number>;
  format?: string;
  dataType?: string;
  byteOrder?: string;
  functionCode?: string;
  raw?: boolean;
  writable?: boolean;
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

const isCoil = computed(() => {
  return props.functionCode === '0x01' || props.functionCode === '0x02';
});

const is32Bit = computed(() => {
  return !isCoil.value && (props.dataType === 'Float32' || props.dataType === 'Int32' || props.dataType === 'UInt32');
});

watch(() => [props.writable, props.startAddress, props.count, props.dataType, props.functionCode, props.format, props.byteOrder], () => {
  showEditModal.value = false;
});

const activeFormat = computed(() => (isCoil.value ? 'Dec' : props.format || 'Dec'));

function isEditableAddress(addr: number): boolean {
  const target = is32Bit.value && (addr - props.startAddress) % 2 !== 0 ? addr - 1 : addr;
  const width = is32Bit.value ? 2 : 1;
  return target >= props.startAddress && target + width <= props.startAddress + props.count;
}

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
  if (addr < props.startAddress || addr >= props.startAddress + props.count) return '-';
  if (isCoil.value) {
    return (props.values[addr] ?? 0) === 1 ? '1' : '0';
  }
  if (is32Bit.value) {
    const offset = addr - props.startAddress;
    if (offset % 2 !== 0 || addr + 1 >= props.startAddress + props.count) {
      return '-';
    }
    const raw1 = props.values[addr] ?? 0;
    const raw2 = props.values[addr + 1] ?? 0;
    return formatRegisterValue(raw1, activeFormat.value, props.dataType, props.byteOrder, raw2);
  }
  return formatRegisterValue(props.values[addr] ?? 0, activeFormat.value, props.dataType, props.byteOrder);
}

function handleModalSave(addr: number, parsed: { word1: number; word2?: number } | number) {
  if (props.writable === false || !isEditableAddress(addr)) return;
  if (typeof parsed === 'object') {
    emit('update-cell-pair', addr, parsed.word1, addr + 1, parsed.word2 ?? 0);
  } else if (!isNaN(parsed)) {
    emit('update-cell', addr, parsed);
  }
}

function copyCell(addr: number) {
  let targetAddr = addr;
  if (is32Bit.value && (targetAddr - props.startAddress) % 2 !== 0) {
    targetAddr = targetAddr - 1;
  }
  const text = getPrimaryCellText(targetAddr);
  navigator.clipboard.writeText(text);
}

function openEditCell(addr: number) {
  if (props.writable === false) return;
  let targetAddr = addr;
  if (is32Bit.value && (targetAddr - props.startAddress) % 2 !== 0) {
    targetAddr = targetAddr - 1;
  }
  modalAddress.value = targetAddr;
  modalCurrentValueText.value = getPrimaryCellText(targetAddr);
  showEditModal.value = true;
}

function toggleCoilCell(addr: number) {
  if (props.writable === false || !isCoil.value) return;
  const current = props.values[addr] ?? 0;
  emit('update-cell', addr, current === 1 ? 0 : 1);
}

function resetCellToZero(addr: number) {
  if (props.writable === false) return;
  let targetAddr = addr;
  if (is32Bit.value && (targetAddr - props.startAddress) % 2 !== 0) {
    targetAddr = targetAddr - 1;
  }
  if (is32Bit.value) {
    emit('update-cell-pair', targetAddr, 0, targetAddr + 1, 0);
  } else {
    emit('update-cell', targetAddr, 0);
  }
}
</script>

<template>
  <div class="flex-1 p-4 overflow-auto bg-gray-50/50 relative">
    <div class="bg-white border border-gray-200 rounded-xl shadow-2xs overflow-hidden">
      <Table class="w-full text-sm text-center border-collapse">
        <TableHeader>
          <TableRow class="bg-gray-50/80 border-b border-gray-200 text-gray-600 font-medium hover:bg-gray-50/80">
            <TableHead class="py-2.5 px-2 w-20 shrink-0 border-r border-gray-200 font-semibold select-none text-center text-gray-600">
              {{ $t('dataGrid.address') }}
            </TableHead>
            <TableHead
              v-for="col in 10"
              :key="col - 1"
              class="py-2.5 px-2 min-w-[70px] whitespace-nowrap border-r border-gray-200 last:border-r-0 select-none text-center font-medium text-gray-600"
            >
              {{ col - 1 }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody class="divide-y divide-gray-200">
          <TableRow
            v-for="(rowAddr, index) in rows"
            :key="rowAddr"
            :class="[
              'transition-colors hover:bg-blue-50/40 border-b border-gray-200',
              index % 2 === 1 ? 'bg-slate-50/60' : 'bg-white'
            ]"
          >
            <TableCell
              :class="[
                'py-2.5 px-2 font-semibold text-gray-700 border-r border-gray-200 font-mono select-none text-center',
                index % 2 === 1 ? 'bg-slate-100/70' : 'bg-gray-50/80'
              ]"
            >
              {{ rowAddr }}
            </TableCell>
            <TableCell
              v-for="colOffset in 10"
              :key="colOffset - 1"
              class="p-0 border-r border-gray-200 last:border-r-0 text-center align-middle font-mono select-none"
            >
              <ContextMenu v-if="isEditableAddress(rowAddr + colOffset - 1)">
                <ContextMenuTrigger as-child>
                  <div class="w-full h-full py-2.5 px-2 cursor-default hover:bg-blue-100/40 text-gray-800 transition-colors flex flex-col items-center justify-center leading-tight">
                    <span>{{ getPrimaryCellText(rowAddr + colOffset - 1) }}</span>
                    <span v-if="props.raw" class="text-[10px] text-gray-400 font-mono tracking-wider font-normal leading-none mt-0.5 select-none">
                      {{ getRawHex(rowAddr + colOffset - 1) }}
                    </span>
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent class="w-40 text-xs">
                  <ContextMenuItem @select="copyCell(rowAddr + colOffset - 1)" class="gap-2 cursor-pointer">
                    <Copy class="w-3.5 h-3.5 text-gray-500" />
                    <span>{{ $t('common.copy') }}</span>
                  </ContextMenuItem>

                  <ContextMenuItem
                    v-if="isCoil"
                    @select="toggleCoilCell(rowAddr + colOffset - 1)"
                    :disabled="writable === false"
                    class="gap-2 cursor-pointer"
                  >
                    <ToggleLeft class="w-3.5 h-3.5 text-gray-500" />
                    <span>切换 (0/1)</span>
                  </ContextMenuItem>

                  <ContextMenuItem
                    @select="openEditCell(rowAddr + colOffset - 1)"
                    :disabled="writable === false"
                    class="gap-2 cursor-pointer"
                  >
                    <Edit3 class="w-3.5 h-3.5 text-gray-500" />
                    <span>{{ $t('common.edit') }}</span>
                  </ContextMenuItem>

                  <ContextMenuSeparator />

                  <ContextMenuItem
                    @select="resetCellToZero(rowAddr + colOffset - 1)"
                    :disabled="writable === false"
                    variant="destructive"
                    class="gap-2 cursor-pointer text-red-600 focus:text-red-600 focus:bg-red-50"
                  >
                    <RotateCcw class="w-3.5 h-3.5 text-red-500" />
                    <span>{{ $t('dataGrid.resetToZero') }}</span>
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>

              <div
                v-else
                class="w-full h-full py-2.5 px-2 cursor-default text-gray-400 flex flex-col items-center justify-center leading-tight"
              >
                <span>-</span>
              </div>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
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
  </div>
</template>
