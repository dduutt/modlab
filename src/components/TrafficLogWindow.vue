<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Trash2, Pin, ArrowDownCircle, Copy, ArrowUpRight, ArrowDownLeft } from '@lucide/vue';

export interface TrafficLogEntry {
  id: string;
  sessionId: string;
  sessionTitle: string;
  direction: 'TX' | 'RX';
  message: string;
  bytes: string;
  timestamp: string;
}

const logs = ref<TrafficLogEntry[]>([]);
const selectedDevice = ref<string>('ALL');
const directionFilter = ref<'ALL' | 'TX' | 'RX'>('ALL');
const isAlwaysOnTop = ref<boolean>(false);
const autoScroll = ref<boolean>(true);
const containerRef = ref<HTMLDivElement | null>(null);

let unlistenFn: UnlistenFn | null = null;

const availableDevices = computed(() => {
  const set = new Set<string>();
  logs.value.forEach(log => set.add(log.sessionTitle));
  return Array.from(set);
});

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    const matchDevice = selectedDevice.value === 'ALL' || log.sessionTitle === selectedDevice.value;
    const matchDir = directionFilter.value === 'ALL' || log.direction === directionFilter.value;
    return matchDevice && matchDir;
  });
});

async function toggleAlwaysOnTop() {
  try {
    const appWindow = getCurrentWebviewWindow();
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await appWindow.setAlwaysOnTop(isAlwaysOnTop.value);
  } catch (err) {
    console.error('Failed to set always on top:', err);
  }
}

function clearLogs() {
  logs.value = [];
}

function copyBytes(bytes: string) {
  navigator.clipboard.writeText(bytes);
}

onMounted(async () => {
  try {
    unlistenFn = await listen<TrafficLogEntry>('traffic-log-entry', (event) => {
      logs.value.unshift(event.payload);
      if (logs.value.length > 500) {
        logs.value.pop();
      }
      if (autoScroll.value) {
        nextTick(() => {
          if (containerRef.value) {
            containerRef.value.scrollTop = 0;
          }
        });
      }
    });
  } catch (err) {
    console.error('Failed to register traffic event listener:', err);
  }
});

onUnmounted(() => {
  if (unlistenFn) {
    unlistenFn();
  }
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-slate-900 text-slate-100 font-sans select-none overflow-hidden">
    <!-- Top Control Bar -->
    <div class="flex items-center justify-between px-4 py-3 bg-slate-800/90 border-b border-slate-700 shadow-md shrink-0">
      <div class="flex items-center gap-3">
        <!-- Device Filter -->
        <div class="flex items-center gap-2">
          <label class="text-xs font-semibold text-slate-400">Device:</label>
          <select
            v-model="selectedDevice"
            class="bg-slate-700 text-slate-200 border border-slate-600 rounded-lg px-3 py-1 text-xs outline-none focus:border-blue-500 font-medium cursor-pointer"
          >
            <option value="ALL">All Devices</option>
            <option v-for="dev in availableDevices" :key="dev" :value="dev">{{ dev }}</option>
          </select>
        </div>

        <!-- Direction Filter Tabs -->
        <div class="flex bg-slate-700/80 p-0.5 rounded-lg text-xs font-medium border border-slate-600">
          <button
            @click="directionFilter = 'ALL'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'ALL' ? 'bg-blue-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            ALL
          </button>
          <button
            @click="directionFilter = 'TX'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'TX' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            TX Only
          </button>
          <button
            @click="directionFilter = 'RX'"
            :class="['px-3 py-1 rounded-md transition cursor-pointer', directionFilter === 'RX' ? 'bg-emerald-600 text-white shadow-xs' : 'text-slate-300 hover:text-white']"
          >
            RX Only
          </button>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2">
        <button
          @click="toggleAlwaysOnTop"
          :class="['p-1.5 rounded-lg border text-xs flex items-center gap-1 font-medium transition cursor-pointer', isAlwaysOnTop ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-700/60 text-slate-300 border-slate-600 hover:bg-slate-700']"
          title="Toggle Always on Top"
        >
          <Pin class="w-3.5 h-3.5" /> {{ isAlwaysOnTop ? 'Pinned' : 'Pin' }}
        </button>

        <button
          @click="autoScroll = !autoScroll"
          :class="['p-1.5 rounded-lg border text-xs flex items-center gap-1 font-medium transition cursor-pointer', autoScroll ? 'bg-blue-500/20 text-blue-300 border-blue-500/40' : 'bg-slate-700/60 text-slate-300 border-slate-600 hover:bg-slate-700']"
          title="Toggle Auto Scroll"
        >
          <ArrowDownCircle class="w-3.5 h-3.5" /> Auto Scroll
        </button>

        <button
          @click="clearLogs"
          class="p-1.5 rounded-lg bg-red-500/20 text-red-300 border border-red-500/40 hover:bg-red-500/30 text-xs flex items-center gap-1 font-medium transition cursor-pointer"
          title="Clear Traffic Logs"
        >
          <Trash2 class="w-3.5 h-3.5" /> Clear
        </button>
      </div>
    </div>

    <!-- Traffic Log List View -->
    <div ref="containerRef" class="flex-1 overflow-y-auto p-3 space-y-2 font-mono text-xs">
      <div v-if="filteredLogs.length === 0" class="flex flex-col items-center justify-center h-full text-slate-500">
        <p>No traffic logs captured.</p>
      </div>

      <div
        v-for="entry in filteredLogs"
        :key="entry.id"
        class="bg-slate-800/60 border border-slate-700/80 rounded-xl p-2.5 flex items-center justify-between gap-3 hover:border-slate-600 transition group"
      >
        <div class="flex items-center gap-3 overflow-hidden">
          <span class="text-slate-400 text-[11px] shrink-0">{{ entry.timestamp }}</span>
          
          <span class="px-2 py-0.5 rounded bg-slate-700 text-slate-200 font-semibold text-[11px] shrink-0 border border-slate-600">
            {{ entry.sessionTitle }}
          </span>

          <span
            :class="[
              'flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-bold shrink-0',
              entry.direction === 'TX' ? 'bg-sky-500/20 text-sky-400 border border-sky-500/30' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
            ]"
          >
            <component :is="entry.direction === 'TX' ? ArrowUpRight : ArrowDownLeft" class="w-3 h-3" />
            {{ entry.direction }}
          </span>

          <span class="text-slate-300 font-medium shrink-0">{{ entry.message }}</span>

          <span class="text-slate-400 truncate tracking-wide text-slate-300 font-mono">{{ entry.bytes }}</span>
        </div>

        <button
          @click="copyBytes(entry.bytes)"
          class="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-slate-200 hover:bg-slate-700 rounded transition cursor-pointer shrink-0"
          title="Copy Bytes"
        >
          <Copy class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>
