<script setup lang="ts">
import { X } from '@lucide/vue';

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
