<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { decodeFrame } from '../utils/logging';
import { Trash2, X, Filter, Copy } from '@lucide/vue';

export interface LogEntry {
  id: string;
  protocol?: 'TCP' | 'RTU';
  role?: 'Master' | 'Slave';
  complete?: boolean;
  sessionId: string;
  timestampMs: number;
  durationMs?: number;
  level: 'info' | 'success' | 'error';
  kind: 'operation' | 'frame';
  detail: boolean;
  time: string;
  deviceName?: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
}

const props = defineProps<{
  logs: LogEntry[];
  devices?: { id: string; title: string }[];
}>();

const emit = defineEmits<{
  (e: 'clear', device?: string): void;
  (e: 'close'): void;
}>();

const selectedDevice = ref<string>('ALL');

const directionFilter = ref<'ALL' | 'TX' | 'RX' | 'ERROR'>('ALL');
const copyFailed = ref(false);
const following = ref(true);
const container = ref<HTMLElement | null>(null);
const frozenLogs = ref<LogEntry[]>([]);
function pause() {
  if (following.value) frozenLogs.value = props.logs.map(log => ({ ...log }));
  following.value = false;
}
async function resume() {
  following.value = true;
  await nextTick();
  if (container.value) container.value.scrollTop = 0;
}
function onScroll(event: Event) {
  if ((event.target as HTMLElement).scrollTop > 4) pause();
}
const displayedLogs = computed(() => following.value ? props.logs : frozenLogs.value);
watch([selectedDevice, directionFilter], resume);
watch(() => props.devices, devices => {
  if (selectedDevice.value !== 'ALL' && !devices?.some(device => device.id === selectedDevice.value)) selectedDevice.value = 'ALL';
});
const filteredLogs = computed(() => displayedLogs.value.filter(log =>
  (selectedDevice.value === 'ALL' || log.sessionId === selectedDevice.value) && (directionFilter.value === 'ALL' || (directionFilter.value === 'ERROR' ? log.level === 'error' : log.kind === 'frame' && log.direction === directionFilter.value))
));
async function copyLog(log: LogEntry) {
  try {
    await navigator.clipboard.writeText(`${log.time} [${log.deviceName || ''}] ${log.kind === 'frame' ? log.direction : 'ERROR'} ${log.bytes}
${decodeFrame(log).join('\n')}`);
    copyFailed.value = false;
  } catch { copyFailed.value = true; }
}
</script>

<template>
  <section id="traffic-log-panel" :aria-label="$t('common.logs')" class="shrink-0 border-t border-gray-200 bg-white select-none">
    <!-- Panel Header -->
    <div class="flex flex-wrap items-center justify-between gap-2 px-4 py-2 bg-gray-50 border-b border-gray-200">
      <div class="flex min-w-0 items-center gap-2 text-xs text-gray-600">
        <span class="font-medium">{{ $t('common.logs') }}</span>
        <span class="text-gray-400 font-mono text-[11px] tabular-nums">
          {{ filteredLogs.length }} / {{ logs.length }}
        </span>

        <!-- Device Name Filter Dropdown -->
        <div class="flex min-w-0 items-center ml-1 gap-1.5 border-l border-gray-200 pl-3">
          <Filter class="w-3 h-3 text-gray-500" />
          <select
            v-model="selectedDevice"
            :aria-label="$t('logWindow.device')"
            class="h-7 min-w-0 max-w-[180px] rounded-md border border-gray-200 bg-white px-2 text-xs text-gray-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
          >
            <option value="ALL">{{ $t('logWindow.allDevices') }}</option>
            <option v-for="device in (devices || [])" :key="device.id" :value="device.id">
              {{ device.title }}
            </option>
          </select>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-2">
        <button @click="following ? pause() : resume()" class="rounded-md px-2 py-1 text-xs text-gray-500 hover:bg-gray-100">
          {{ $t(following ? 'logWindow.pause' : 'logWindow.resume') }}
        </button>
        <select v-model="directionFilter" :aria-label="$t('logWindow.filter')" class="h-7 rounded-md border border-gray-200 bg-white px-2 text-xs text-gray-600">
          <option value="ALL">{{ $t('logWindow.all') }}</option>
          <option value="TX">TX</option>
          <option value="RX">RX</option>
          <option value="ERROR">{{ $t('logWindow.error') }}</option>
        </select>
        <button
          @click="emit('clear', selectedDevice === 'ALL' ? undefined : selectedDevice); frozenLogs = []; following = true"
          class="inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-gray-500 hover:bg-gray-100 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 transition-colors cursor-pointer"
          :title="$t('common.clear')"
        >
          <Trash2 class="w-3.5 h-3.5" />
          {{ $t('common.clear') }}
        </button>
        <button
          @click="emit('close')"
          class="inline-flex h-7 w-7 items-center justify-center rounded-md text-gray-400 hover:bg-gray-100 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 transition-colors cursor-pointer"
          :title="$t('common.close')"
          :aria-label="$t('common.close')"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <p v-if="copyFailed" role="alert" class="px-4 py-1 text-xs text-red-600">{{ $t('logWindow.copyFailed') }}</p>
    <!-- Content Panel (Plain text theme) -->
    <div ref="container" @scroll="onScroll" class="h-44 overflow-auto [overflow-anchor:none] px-4 py-1 font-mono text-[11px] bg-white text-gray-700 leading-6">
      <div v-if="filteredLogs.length === 0" class="flex h-full items-center justify-center text-gray-400 font-sans text-xs">
        {{ $t('logWindow.noLogs') }}
      </div>
      <details v-for="log in filteredLogs" :key="log.id" @toggle="(event: Event) => { if ((event.target as HTMLDetailsElement).open) pause(); }" class="min-w-[540px] border-b border-gray-100 last:border-0">
        <summary class="flex cursor-pointer list-none items-baseline gap-3 py-1 hover:bg-gray-50 focus-visible:outline-blue-400">
          <span class="w-[92px] shrink-0 text-gray-400">{{ log.time }}</span>
          <span class="w-16 shrink-0 truncate text-gray-500" :title="log.deviceName">{{ log.deviceName }}</span>
          <span :class="['w-12 shrink-0', log.level === 'error' ? 'text-red-600' : log.level === 'success' ? 'text-emerald-600' : 'text-gray-500']">
            {{ log.kind === 'frame' ? log.direction : $t('logWindow.' + log.level) }}
          </span>
          <span :class="['min-w-0 flex-1 whitespace-pre-wrap break-all select-text font-mono', log.level === 'error' ? 'text-red-600' : 'text-gray-700']" :title="log.message + ' ' + log.bytes">{{ log.bytes }}</span>
          <span v-if="log.durationMs !== undefined" class="shrink-0 text-gray-400">{{ log.durationMs }} ms</span>
          <button @click.stop.prevent="copyLog(log)" :title="$t('common.copy')" :aria-label="$t('common.copy')" class="shrink-0 rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-700"><Copy class="h-3.5 w-3.5" /></button>
        </summary>
        <div class="select-text whitespace-pre-wrap break-all rounded bg-gray-50 px-3 py-2 text-gray-600">
          <div v-for="(line, index) in decodeFrame(log)" :key="index">{{ line }}</div>
        </div>
      </details>
    </div>
  </section>
</template>
