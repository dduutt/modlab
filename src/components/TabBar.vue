<script setup lang="ts">
import { Plus, X } from '@lucide/vue';

interface Tab {
  id: string;
  title: string;
  connected?: boolean;
  busy?: boolean;
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

function getTabRole(tab: Tab): 'Slave' | 'Master' {
  if (tab.connection?.role) {
    return tab.connection.role;
  }
  return tab.title.toLowerCase().includes('master') ? 'Master' : 'Slave';
}
</script>

<template>
  <div class="flex shrink-0 overflow-x-auto overflow-y-hidden items-center bg-gray-100/75 px-3 pt-1.5 border-b border-gray-200 gap-1.5 select-none scrollbar-none">
    <div
      v-for="tab in props.tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      :title="tab.title"
      :class="[
        'flex shrink-0 items-center gap-2 px-3 py-1.5 border rounded-t-md text-xs font-medium transition-all cursor-pointer group relative -mb-px',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-2xs z-10'
          : 'bg-gray-50/70 border-gray-200/80 border-b-gray-200 text-gray-500 hover:bg-gray-100/90 hover:text-gray-700'
      ]"
    >
      <!-- Connection Indicator: Green only when connected, neutral gray when disconnected -->
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0 transition-colors',
          tab.connected ? 'bg-emerald-500 shadow-xs ring-2 ring-emerald-100' : 'bg-gray-300'
        ]"
      ></span>

      <!-- Master / Slave Micro-Badge -->
      <span
        :class="[
          'text-[10px] font-semibold px-1.5 py-0.5 rounded leading-none uppercase tracking-wide shrink-0 transition-colors',
          getTabRole(tab) === 'Master'
            ? tab.id === activeTabId
              ? 'bg-blue-100 text-blue-700 border border-blue-200/80'
              : 'bg-blue-50 text-blue-600/80 border border-blue-100'
            : tab.id === activeTabId
              ? 'bg-amber-100 text-amber-800 border border-amber-200/80'
              : 'bg-amber-50 text-amber-700/80 border border-amber-100'
        ]"
      >
        {{ getTabRole(tab) }}
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
      class="shrink-0 p-1.5 mb-px text-gray-500 hover:text-gray-800 hover:bg-gray-200/60 rounded-md transition cursor-pointer"
      :title="$t ? $t('modal.addDevice') : 'Add Device'"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
