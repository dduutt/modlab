<script setup lang="ts">
import { Terminal } from '@lucide/vue';

defineProps<{
  message: string;
  type: 'error' | 'success' | 'info';
  logCount?: number;
  showLogPanel?: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-logs'): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-1.5 bg-white border-t border-gray-200 text-xs select-none min-h-[36px]">
    <!-- Left: Status Indicator & Message -->
    <div class="flex items-center gap-2">
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0',
          type === 'error' ? 'bg-red-500 animate-pulse' : type === 'success' ? 'bg-emerald-500' : 'bg-blue-500'
        ]"
      ></span>
      <span
        :class="[
          'font-medium truncate max-w-md',
          type === 'error' ? 'text-red-600' : type === 'success' ? 'text-emerald-700' : 'text-gray-600'
        ]"
      >
        {{ message }}
      </span>
    </div>

    <!-- Right: Logs Toggle Button & Copyright -->
    <div class="flex items-center gap-3">
      <button
        @click="emit('toggle-logs')"
        :class="[
          'flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border transition cursor-pointer',
          showLogPanel
            ? 'bg-blue-50 text-blue-600 border-blue-200 shadow-2xs font-semibold'
            : 'bg-gray-100 text-gray-700 border-gray-200 hover:bg-gray-200'
        ]"
        title="Toggle Traffic Log Panel"
      >
        <Terminal class="w-3.5 h-3.5 text-gray-500" />
        <span>Logs</span>
        <span
          :class="[
            'px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold',
            showLogPanel ? 'bg-blue-100 text-blue-700' : 'bg-gray-200 text-gray-700'
          ]"
        >
          {{ logCount || 0 }}
        </span>
      </button>

      <div class="text-gray-400 font-mono text-[11px] hidden sm:block select-all">
        © Modlab • dote27@163.com
      </div>
    </div>
  </div>
</template>
