<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { Users, ChevronDown } from '@lucide/vue';
import { ModbusService, type TcpClientInfo } from '../services/modbusService';

const props = defineProps<{ sessionId: string }>();
const root = ref<HTMLDetailsElement | null>(null);
const clients = ref<TcpClientInfo[] | null>(null);
const failed = ref(false);
const position = ref({ left: '24px', top: '0px' });
function placePanel() {
  if (!root.value?.open) return;
  const rect = root.value.getBoundingClientRect();
  const width = Math.min(480, window.innerWidth - 48);
  position.value = { left: `${Math.max(24, Math.min(rect.left, window.innerWidth - width - 24))}px`, top: `${rect.bottom + 8}px` };
}
let active = true;
let timer: ReturnType<typeof setTimeout> | undefined;

async function refresh() {
  try {
    const result = await ModbusService.listTcpClients(props.sessionId);
    if (!active) return;
    clients.value = result;
    failed.value = false;
  } catch {
    if (active) failed.value = true;
  } finally {
    if (active) timer = setTimeout(refresh, 1000);
  }
}
function closeOutside(event: PointerEvent) {
  if (root.value && !root.value.contains(event.target as Node)) root.value.open = false;
}
function formatTime(timestamp: number | null) {
  return timestamp === null ? '—' : new Date(timestamp).toLocaleString();
}
onMounted(() => {
  void refresh();
  document.addEventListener('pointerdown', closeOutside);
  window.addEventListener('resize', placePanel);
});
onUnmounted(() => {
  active = false;
  clearTimeout(timer);
  document.removeEventListener('pointerdown', closeOutside);
  window.removeEventListener('resize', placePanel);
});
</script>

<template>
  <details ref="root" @toggle="placePanel" class="relative" @keydown.esc="root && (root.open = false)">
    <summary class="flex h-7 cursor-pointer list-none items-center gap-1.5 rounded-md px-2 text-xs text-gray-500 transition-colors hover:bg-gray-50 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400" :title="$t('clients.scope')">
      <Users class="h-3.5 w-3.5" />
      <span>{{ $t('clients.title') }}</span>
      <span class="min-w-[1ch] text-right tabular-nums">{{ failed ? '—' : clients === null ? '…' : clients.length }}</span>
      <ChevronDown class="h-3 w-3 text-gray-400" />
    </summary>
    <div :style="position" class="fixed z-40 w-[min(480px,calc(100vw-48px))] rounded-lg border border-gray-200 bg-white shadow-lg">
      <div class="max-h-64 overflow-auto rounded-lg">
        <table class="w-full text-left text-[11px]">
          <thead class="sticky top-0 bg-gray-50 text-gray-500">
            <tr><th class="px-4 py-2 font-medium">{{ $t('clients.address') }}</th><th class="px-2 py-2 font-medium">{{ $t('clients.connected') }}</th><th class="px-2 py-2 font-medium">{{ $t('clients.lastRequest') }}</th></tr>
          </thead>
          <tbody class="divide-y divide-gray-100 text-gray-600">
            <tr v-if="failed || clients === null || clients.length === 0">
              <td colspan="3" role="status" :class="['px-4 py-5 text-xs', failed ? 'text-red-600' : 'text-gray-400']">
                {{ $t(failed ? 'clients.failed' : clients === null ? 'clients.loading' : 'clients.empty') }}
              </td>
            </tr>
            <tr v-for="client in (failed ? [] : clients || [])" :key="client.id">
              <td class="whitespace-nowrap px-4 py-2 font-mono select-text">{{ client.address }}</td>
              <td class="px-2 py-2 tabular-nums">{{ formatTime(client.connectedAt) }}</td>
              <td class="px-2 py-2 tabular-nums">{{ formatTime(client.lastRequestAt) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </details>
</template>
