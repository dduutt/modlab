<script setup lang="ts">
import { Plus, X } from '@lucide/vue';

interface Tab {
  id: string;
  title: string;
  connected?: boolean;
  busy?: boolean;
  statusType?: 'error' | 'success' | 'info';
  statusMessage?: string;
  connection?: {
    role?: 'Slave' | 'Master';
    protocol?: string;
  };
}

const props = defineProps<{
  tabs: Tab[];
  activeTabId: string;
}>();

const emit = defineEmits<{
  (e: 'select-tab', id: string): void;
  (e: 'add-tab'): void;
  (e: 'close-tab', id: string): void;
}>();

function getTabRoleShort(tab: Tab): 'S' | 'M' {
  const role = tab.connection?.role || (tab.title.toLowerCase().includes('master') ? 'Master' : 'Slave');
  return role === 'Master' ? 'M' : 'S';
}

function getTabStatusColorClass(tab: Tab): string {
  if (tab.statusType === 'error') {
    return 'text-red-500';
  }
  if (tab.connected) {
    return 'text-emerald-600';
  }
  return tab.id === props.activeTabId
    ? 'text-gray-700'
    : 'text-gray-400 group-hover:text-gray-500';
}

function getTabRoleTooltip(tab: Tab): string {
  const role = getTabRoleShort(tab) === 'M' ? 'Master' : 'Slave';
  return tab.statusMessage ? `${role} · ${tab.statusMessage}` : role;
}
</script>

<template>
  <div class="flex shrink-0 overflow-x-auto overflow-y-hidden items-center bg-white px-3 pt-2 border-b border-gray-200 gap-1.5 select-none scrollbar-none">
    <div
      v-for="tab in props.tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      :title="tab.title"
      :class="[
        'flex shrink-0 items-center gap-2 px-3.5 py-1.5 border rounded-t-lg text-xs font-medium transition-all cursor-pointer group relative -mb-px',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-2xs z-10'
          : 'bg-gray-50 border-gray-200 text-gray-500 hover:bg-gray-100 hover:text-gray-700'
      ]"
    >
      <!-- S / M Role Indicator: Pure mono character, color dynamically tracks latest status -->
      <span
        :class="[
          'font-mono text-sm font-bold shrink-0 transition-colors leading-none',
          getTabStatusColorClass(tab)
        ]"
        :title="getTabRoleTooltip(tab)"
      >
        {{ getTabRoleShort(tab) }}
      </span>

      <!-- Tab Title -->
      <span class="truncate max-w-[130px] font-medium leading-tight text-gray-700 group-hover:text-gray-900 transition-colors">
        {{ tab.title }}
      </span>

      <!-- Close Tab Button -->
      <button
        v-if="props.tabs.length > 1"
        :disabled="tab.busy"
        @click.stop="emit('close-tab', tab.id)"
        class="opacity-0 group-hover:opacity-100 p-0.5 -mr-1 hover:bg-gray-200/70 active:bg-gray-300 rounded transition-all cursor-pointer text-gray-400 hover:text-gray-600 focus:opacity-100"
        :title="$t ? $t('common.close') : 'Close'"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Add Tab Button -->
    <button
      @click="emit('add-tab')"
      class="shrink-0 p-1.5 mb-px text-gray-500 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition cursor-pointer"
      :title="$t ? $t('modal.addDevice') : 'Add Device'"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
