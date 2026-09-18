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
  <Transition name="modal">
    <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
      <div role="dialog" aria-modal="true" class="modal-card bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden">
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
          <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">{{ $t('common.cancel') }}</button>
          <button @click="emit('confirm')" class="px-5 py-1.5 rounded-xl bg-red-600 hover:bg-red-700 text-white font-medium shadow-md transition text-sm cursor-pointer">{{ $t('common.confirm') }}</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-card {
  animation: modal-zoom-in 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-leave-active .modal-card {
  animation: modal-zoom-out 0.15s cubic-bezier(0.4, 0, 1, 1);
}

@keyframes modal-zoom-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes modal-zoom-out {
  from {
    opacity: 1;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(0.96);
  }
}
</style>
