<script setup lang="ts">
import { ref, nextTick } from 'vue';
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
  (e: 'rename-tab', id: string, newTitle: string): void;
}>();

const editingId = ref<string | null>(null);
const editTitle = ref<string>('');
const inputRef = ref<HTMLInputElement | null>(null);

async function startRename(tab: Tab) {
  editingId.value = tab.id;
  editTitle.value = tab.title;
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
}

function saveRename(id: string) {
  if (editingId.value !== id) return;
  const trimmed = editTitle.value.trim();
  if (trimmed) {
    const isDuplicate = props.tabs.some(
      t => t.id !== id && t.title.toLowerCase() === trimmed.toLowerCase()
    );
    if (!isDuplicate) {
      emit('rename-tab', id, trimmed);
    }
  }
  editingId.value = null;
}
</script>

<template>
  <div class="flex shrink-0 overflow-x-auto overflow-y-hidden items-center bg-white px-4 pt-2 border-b border-gray-200 gap-2 select-none">
    <div
      v-for="tab in props.tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      @dblclick="startRename(tab)"
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
      
      <template v-if="editingId === tab.id">
        <input
          :ref="el => { inputRef = el as HTMLInputElement | null; }"
          v-model="editTitle"
          @blur="saveRename(tab.id)"
          @keyup.enter="saveRename(tab.id)"
          @keyup.esc="editingId = null"
          @click.stop
          class="px-1.5 py-0.5 border border-blue-500 rounded bg-white outline-none font-medium text-sm text-gray-900 w-24"
        />
      </template>
      <template v-else>
        <span>{{ tab.title }}</span>
      </template>

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
