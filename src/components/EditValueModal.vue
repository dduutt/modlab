<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { X } from '@lucide/vue';
import { parseFormattedRegisterValue } from '../utils/modbusFormatter';

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

const maxLength = computed(() => {
  if (props.format === 'Hex') {
    return props.is32Bit ? 10 : 6; // 0xFFFFFFFF or 0xFFFF
  }
  if (props.dataType === 'Float32') {
    return 15;
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

  if (props.format === 'Hex') {
    val = val.replace(/[^0-9a-fA-F--xX]/g, '');
  } else if (props.dataType === 'Float32') {
    val = val.replace(/[^0-9.-]/g, '');
  } else if (props.dataType === 'Int16' || props.dataType === 'Int32') {
    val = val.replace(/[^0-9-]/g, '');
  } else {
    val = val.replace(/[^0-9]/g, '');
  }

  if (val.length > maxLength.value) {
    val = val.slice(0, maxLength.value);
  }

  newValueInput.value = val;
}

function handleSave() {
  if (props.address === null || !newValueInput.value.trim()) {
    emit('close');
    return;
  }

  const parsed = parseFormattedRegisterValue(
    newValueInput.value,
    props.format,
    props.dataType,
    props.byteOrder
  );

  emit('save', props.address, parsed);
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-xs overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <h3 class="font-semibold text-gray-900 text-sm">Edit Register Value</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-4 space-y-3 text-xs">
        <div class="flex items-center justify-between bg-gray-50 px-3 py-2 rounded-xl border border-gray-100">
          <span class="text-gray-500 font-medium">Address</span>
          <span class="font-mono font-bold text-gray-900">
            {{ address !== null ? (is32Bit ? `${address} - ${address + 1}` : address) : '-' }}
          </span>
        </div>

        <div class="flex items-center justify-between bg-gray-50 px-3 py-2 rounded-xl border border-gray-100">
          <span class="text-gray-500 font-medium">Format / Type</span>
          <span class="font-mono text-blue-600 font-semibold">
            {{ dataType }} ({{ format }})
          </span>
        </div>

        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1">New Value</label>
          <input
            ref="inputRef"
            :value="newValueInput"
            :maxlength="maxLength"
            @input="handleInputFilter"
            @keyup.enter="handleSave"
            @keyup.esc="emit('close')"
            type="text"
            class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-mono text-sm font-semibold bg-white text-gray-900 shadow-2xs"
          />
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end gap-2 px-4 py-3 bg-gray-50/80 border-t border-gray-100">
        <button
          @click="emit('close')"
          class="px-3.5 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-xs cursor-pointer"
        >
          Cancel
        </button>
        <button
          @click="handleSave"
          class="px-4 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-xs transition text-xs cursor-pointer"
        >
          Save
        </button>
      </div>
    </div>
  </div>
</template>
