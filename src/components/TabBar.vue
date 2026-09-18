<script setup lang="ts">
import { Plus, X } from '@lucide/vue';

interface Tab {
  id: string;
  title: string;
  connected?: boolean;
  busy?: boolean;
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
</script>

<template>
  <div class="flex shrink-0 overflow-x-auto overflow-y-hidden items-center bg-white px-4 pt-2 border-b border-gray-200 gap-2 select-none">
    <div
      v-for="tab in props.tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      :class="[
        'flex shrink-0 items-center gap-2 px-4 py-2 border rounded-t-lg text-sm font-medium transition-all cursor-pointer group',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-2xs z-10'
          : 'bg-gray-50 border-gray-200 text-gray-500 hover:bg-gray-100 hover:text-gray-700'
      ]"
    >
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0 transition-colors',
          tab.connected ? 'bg-emerald-500 shadow-xs' : tab.id === activeTabId ? 'bg-blue-500' : 'bg-gray-300'
        ]"
      ></span>
      
      <span>{{ tab.title }}</span>

      <button
        v-if="props.tabs.length > 1"
        :disabled="tab.busy"
        @click.stop="emit('close-tab', tab.id)"
        class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-gray-200/80 rounded transition cursor-pointer"
      >
        <X class="w-3.5 h-3.5 text-gray-400 hover:text-gray-600" />
      </button>
    </div>

    <button
      @click="emit('add-tab')"
      class="shrink-0 p-1.5 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition cursor-pointer"
      :title="$t('modal.addDevice')"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
