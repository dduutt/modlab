<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { parseFormattedRegisterValue } from '../utils/modbusFormatter';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

const props = defineProps<{
  show: boolean;
  address: number | null;
  currentValueText: string;
  format: string;
  dataType: string;
  byteOrder: string;
  is32Bit: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', address: number, parsedResult: { word1: number; word2?: number } | number): void;
}>();

const newValueInput = ref<string>('');
const inputRef = ref<HTMLInputElement | null>(null);
const invalidValue = ref(false);

const maxLength = computed(() => {
  if (props.format === 'Hex') {
    return props.is32Bit ? 10 : 6; // 0xFFFFFFFF or 0xFFFF
  }
  if (props.dataType === 'Float32') {
    return 32;
  }
  if (props.dataType === 'Int32') {
    return 11; // -2147483648
  }
  if (props.dataType === 'UInt32') {
    return 10; // 4294967295
  }
  if (props.dataType === 'Int16') {
    return 6; // -32768
  }
  return 5; // 65535
});

watch(
  () => props.show,
  async (isShowing) => {
    if (isShowing) {
      invalidValue.value = false;
      newValueInput.value = props.currentValueText;
      await nextTick();
      inputRef.value?.focus();
      inputRef.value?.select();
    }
  },
  { immediate: true }
);

function handleInputFilter(e: Event) {
  const target = e.target as HTMLInputElement;
  let val = target.value;
  invalidValue.value = false;

  if (props.format === 'Hex') {
    val = val.replace(/[^0-9a-fA-FxX]/g, '');
  } else if (props.dataType === 'Float32') {
    val = val.replace(/[^0-9.eE+-]/g, '');
  } else if (props.dataType === 'Int16' || props.dataType === 'Int32') {
    val = val.replace(/[^0-9-]/g, '');
  } else {
    val = val.replace(/[^0-9]/g, '');
  }

  if (val.length > maxLength.value) {
    val = val.slice(0, maxLength.value);
  }

  newValueInput.value = val;
  target.value = val;
}

function handleSave() {
  if (props.address === null || !newValueInput.value.trim()) {
    emit('close');
    return;
  }

  try {
    const parsed = parseFormattedRegisterValue(
      newValueInput.value,
      props.format,
      props.dataType,
      props.byteOrder
    );

    emit('save', props.address, parsed);
    emit('close');
  } catch {
    invalidValue.value = true;
  }
}
</script>

<template>
  <Dialog :open="show" @update:open="(val: boolean) => { if (!val) emit('close'); }">
    <DialogContent class="max-w-xs rounded-xl p-5 select-none bg-white">
      <DialogHeader class="space-y-1">
        <DialogTitle class="text-sm font-semibold text-gray-900">
          {{ $t('dataGrid.editRegisterValue') }}
        </DialogTitle>
      </DialogHeader>

      <div class="space-y-2.5 text-xs py-2">
        <div class="flex items-center justify-between bg-gray-50 px-3 py-1.5 rounded-lg border border-gray-100">
          <span class="text-gray-500 font-medium">{{ $t('dataGrid.address') }}</span>
          <span class="font-mono font-bold text-gray-900">
            {{ address !== null ? (is32Bit ? `${address} - ${address + 1}` : address) : '-' }}
          </span>
        </div>

        <div class="flex items-center justify-between bg-gray-50 px-3 py-1.5 rounded-lg border border-gray-100">
          <span class="text-gray-500 font-medium">{{ $t('dataGrid.formatType') }}</span>
          <span class="font-mono text-blue-600 font-semibold">
            {{ dataType }} ({{ format }})
          </span>
        </div>

        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('dataGrid.newValue') }}</label>
          <input
            ref="inputRef"
            :value="newValueInput"
            :maxlength="maxLength"
            @input="handleInputFilter"
            @keyup.enter="handleSave"
            type="text"
            class="w-full px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 font-mono text-sm font-semibold bg-white text-gray-900 shadow-2xs"
          />
          <p v-if="invalidValue" role="alert" class="mt-1.5 text-xs text-red-600 font-medium">{{ $t('dataGrid.invalidValue') }}</p>
        </div>
      </div>

      <DialogFooter class="mt-2 gap-2">
        <button
          type="button"
          @click="emit('close')"
          class="px-3.5 py-1.5 rounded-lg border border-gray-200 text-gray-600 hover:bg-gray-50 font-medium transition text-xs cursor-pointer"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="button"
          @click="handleSave"
          class="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-xs transition text-xs cursor-pointer"
        >
          {{ $t('common.save') }}
        </button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
