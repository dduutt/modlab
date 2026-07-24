<script setup lang="ts">
import { ref, computed } from 'vue';
import { Trash2, ArrowUpRight, ArrowDownLeft, X, Filter } from '@lucide/vue';

export interface LogEntry {
  id: string;
  time: string;
  deviceName?: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
}

const props = defineProps<{
  logs: LogEntry[];
  deviceNames?: string[];
}>();

const emit = defineEmits<{
  (e: 'clear'): void;
  (e: 'close'): void;
}>();

const selectedDevice = ref<string>('ALL');

const filteredLogs = computed(() => {
  if (selectedDevice.value === 'ALL') return props.logs;
  return props.logs.filter(l => l.deviceName === selectedDevice.value);
});
</script>

<template>
  <div class="border-t border-gray-200 bg-white select-none shadow-xl animate-in slide-in-from-bottom-2 duration-150">
    <!-- Panel Header -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-50/90 border-b border-gray-200">
      <div class="flex items-center gap-3 text-xs font-semibold text-gray-700">
        <span>Traffic Log</span>
        <span class="px-2 py-0.5 rounded-full bg-gray-200/70 text-gray-600 font-mono text-[10px]">
          {{ filteredLogs.length }} / {{ logs.length }} entries
        </span>

        <!-- Device Name Filter Dropdown -->
        <div class="flex items-center gap-1.5 ml-2 bg-white px-2.5 py-1 rounded-lg border border-gray-200 text-xs font-normal">
          <Filter class="w-3.5 h-3.5 text-gray-400" />
          <span class="text-gray-500 font-medium text-xs">Device:</span>
          <select
            v-model="selectedDevice"
            class="bg-transparent outline-none text-xs text-gray-700 font-medium cursor-pointer"
          >
            <option value="ALL">All Devices</option>
            <option v-for="name in (deviceNames || [])" :key="name" :value="name">
              {{ name }}
            </option>
          </select>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          @click="emit('clear')"
          class="flex items-center gap-1 px-2.5 py-1 text-xs text-gray-600 hover:text-red-600 hover:bg-red-50 rounded-full transition cursor-pointer"
          title="Clear traffic log"
        >
          <Trash2 class="w-3.5 h-3.5" />
          Clear
        </button>
        <button
          @click="emit('close')"
          class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-full transition cursor-pointer"
          title="Close log panel"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Content Panel (Soft Light Theme) -->
    <div class="h-44 overflow-auto p-3 font-mono text-xs bg-slate-50/70 text-gray-700 space-y-1.5 divide-y divide-gray-200/60">
      <div v-if="filteredLogs.length === 0" class="text-gray-400 text-center py-8 italic font-sans text-xs">
        {{ logs.length === 0 ? 'No traffic recorded yet. Connect to start sending/receiving Modbus frames.' : `No logs for device "${selectedDevice}".` }}
      </div>
      <div v-for="log in filteredLogs" :key="log.id" class="flex items-center gap-3 pt-1.5 first:pt-0">
        <span class="text-gray-400 text-[11px] select-none shrink-0 font-mono">{{ log.time }}</span>
        
        <!-- Device Name Badge -->
        <span v-if="log.deviceName" class="text-gray-600 text-[10px] bg-gray-100 px-1.5 py-0.5 rounded border border-gray-200 font-sans font-medium shrink-0">
          {{ log.deviceName }}
        </span>

        <span
          :class="[
            'px-2 py-0.5 rounded-full text-[10px] font-bold flex items-center gap-1 shrink-0 border',
            log.direction === 'TX'
              ? 'bg-blue-50 text-blue-600 border-blue-200'
              : 'bg-emerald-50 text-emerald-600 border-emerald-200'
          ]"
        >
          <component :is="log.direction === 'TX' ? ArrowUpRight : ArrowDownLeft" class="w-3 h-3" />
          {{ log.direction }}
        </span>
        <span class="text-gray-700 font-semibold shrink-0 text-xs">{{ log.message }}</span>
        <span class="text-gray-600 bg-white px-2 py-0.5 rounded border border-gray-200/80 font-mono text-[11px] break-all select-all">
          {{ log.bytes }}
        </span>
      </div>
    </div>
  </div>
</template>
