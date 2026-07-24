# Enhancement Task 3 Brief: Traffic Log Drawer Component (`TrafficLog.vue`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS

## Instructions
Create `src/components/TrafficLog.vue` for logging and inspecting Modbus TX/RX frames.

Code for `src/components/TrafficLog.vue`:
```vue
<script setup lang="ts">
import { ref } from 'vue';
import { ChevronUp, ChevronDown, Trash2, ArrowUpRight, ArrowDownLeft } from '@lucide/vue';

export interface LogEntry {
  id: string;
  time: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
}

defineProps<{
  logs: LogEntry[];
}>();

const emit = defineEmits<{
  (e: 'clear'): void;
}>();

const isOpen = ref<boolean>(false);
</script>

<template>
  <div class="border-t border-gray-200 bg-white select-none transition-all">
    <!-- Header bar -->
    <div
      @click="isOpen = !isOpen"
      class="flex items-center justify-between px-4 py-2 bg-gray-50/80 hover:bg-gray-100/80 cursor-pointer border-b border-gray-200/50"
    >
      <div class="flex items-center gap-2 text-xs font-semibold text-gray-700">
        <span>Traffic Log</span>
        <span class="px-2 py-0.5 rounded-full bg-gray-200 text-gray-700 font-mono text-[10px]">{{ logs.length }} entries</span>
      </div>

      <div class="flex items-center gap-3" @click.stop>
        <button @click="emit('clear')" class="p-1 text-gray-400 hover:text-red-600 rounded transition cursor-pointer" title="Clear log">
          <Trash2 class="w-3.5 h-3.5" />
        </button>
        <button @click="isOpen = !isOpen" class="p-1 text-gray-500 hover:text-gray-700 rounded transition cursor-pointer">
          <component :is="isOpen ? ChevronDown : ChevronUp" class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Content Panel -->
    <div v-if="isOpen" class="h-40 overflow-auto p-3 font-mono text-xs bg-gray-900 text-gray-200 space-y-1.5">
      <div v-if="logs.length === 0" class="text-gray-500 text-center py-4 italic">No traffic recorded yet. Connect to start polling.</div>
      <div v-for="log in logs" :key="log.id" class="flex items-start gap-3 py-0.5 border-b border-gray-800/50 last:border-b-0">
        <span class="text-gray-500 select-none">{{ log.time }}</span>
        <span
          :class="[
            'px-1.5 py-0.5 rounded text-[10px] font-bold flex items-center gap-1 shrink-0',
            log.direction === 'TX' ? 'bg-blue-900/60 text-blue-300 border border-blue-700/50' : 'bg-emerald-900/60 text-emerald-300 border border-emerald-700/50'
          ]"
        >
          <component :is="log.direction === 'TX' ? ArrowUpRight : ArrowDownLeft" class="w-3 h-3" />
          {{ log.direction }}
        </span>
        <span class="text-gray-300 font-semibold shrink-0">{{ log.message }}</span>
        <span class="text-amber-300/90 break-all select-all font-mono">{{ log.bytes }}</span>
      </div>
    </div>
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/enhancement-task-3-report.md`.
