<script setup lang="ts">
import { Terminal, Languages } from '@lucide/vue';
import { ref, watch, computed } from 'vue';

const props = defineProps<{
  message: string;
  type: 'error' | 'success' | 'info';
  logCount?: number;
  showLogPanel?: boolean;
  language?: 'en' | 'zh';
}>();

const emit = defineEmits<{
  (e: 'toggle-logs'): void;
  (e: 'toggle-language'): void;
}>();

const timestamp = ref(new Date().toLocaleTimeString());

watch(() => props.message, () => {
  timestamp.value = new Date().toLocaleTimeString();
});

const conciseMessage = computed(() => {
  if (!props.message) return '';
  return props.message.split('\n')[0].substring(0, 150);
});
</script>

<template>
  <div class="flex items-center justify-between gap-3 px-4 py-1 bg-white border-t border-gray-200 text-xs select-none min-h-[36px] shrink-0">
    <!-- Left: Status Indicator & Message -->
    <div class="flex min-w-0 flex-1 items-center gap-2">
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0',
          type === 'error' ? 'bg-red-500 animate-pulse' : type === 'success' ? 'bg-emerald-500' : 'bg-blue-500'
        ]"
      ></span>
      <span
        :class="[
          'font-medium truncate',
          type === 'error' ? 'text-red-600' : type === 'success' ? 'text-emerald-700' : 'text-gray-600'
        ]"
        :title="message"
      >
        <span class="opacity-70 font-mono text-[11px] mr-1.5">[{{ timestamp }}]</span>
        {{ conciseMessage }}
      </span>
    </div>

    <!-- Right: Logs Toggle Button & Copyright -->
    <div class="flex shrink-0 items-center gap-1">
      <button
        @click="emit('toggle-language')"
        class="inline-flex h-7 min-w-[60px] items-center justify-center gap-1.5 rounded-md px-2 text-xs text-gray-500 transition-colors hover:bg-gray-50 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
        :title="language === 'zh' ? 'Switch to English' : '切换为中文'"
        :aria-label="language === 'zh' ? 'Switch to English' : '切换为中文'"
      >
        <Languages class="w-3.5 h-3.5 text-gray-500" />
        <span>{{ language === 'zh' ? '中' : 'EN' }}</span>
      </button>

      <button
        @click="emit('toggle-logs')"
        :class="[
          'inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-xs transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer',
          showLogPanel
            ? 'bg-gray-100 text-gray-900'
            : 'text-gray-500 hover:bg-gray-50 hover:text-gray-900'
        ]"
        :title="$t('common.logs')"
        :aria-expanded="!!showLogPanel"
        aria-controls="traffic-log-panel"
      >
        <Terminal class="w-3.5 h-3.5 text-gray-500" />
        <span>{{ $t('common.logs') }}</span>
        <span class="min-w-[2ch] text-right font-mono text-[10px] tabular-nums text-gray-400">
          {{ logCount || 0 }}
        </span>
      </button>

      <div class="ml-2 border-l border-gray-200 pl-3 text-gray-400 text-[11px] whitespace-nowrap select-all">
        © Modlab • dote27@163.com
      </div>
    </div>
  </div>
</template>
