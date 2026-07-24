# Task 2 Brief: Tab Inline Renaming & Connection Safety Guard Modal

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS

## Instructions
1. Create `src/components/ConfirmModal.vue` warning confirmation modal.
2. Update `src/components/TabBar.vue` to allow double-clicking tab text to edit title inline (emitting `rename-tab(id, newTitle)`).
3. Update `src/App.vue` to show `ConfirmModal` before closing a tab if `connected === true`.

Code for `src/components/ConfirmModal.vue`:
```vue
<script setup lang="ts">
import { AlertTriangle, X } from '@lucide/vue';

defineProps<{
  show: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'confirm'): void;
}>();
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-red-50/50">
        <div class="flex items-center gap-2 text-red-600">
          <AlertTriangle class="w-5 h-5" />
          <h3 class="font-semibold text-gray-900 text-base">{{ title }}</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 text-sm text-gray-600">
        {{ message }}
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="emit('confirm')" class="px-5 py-1.5 rounded-xl bg-red-600 hover:bg-red-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Confirm Close</button>
      </div>
    </div>
  </div>
</template>
```

Code for `src/components/TabBar.vue`:
```vue
<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { Plus, X } from '@lucide/vue';

interface Tab {
  id: string;
  title: string;
}

defineProps<{
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
  const trimmed = editTitle.value.trim();
  if (trimmed) {
    emit('rename-tab', id, trimmed);
  }
  editingId.value = null;
}
</script>

<template>
  <div class="flex items-center bg-white px-4 pt-2 border-b border-gray-200 gap-2 select-none">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      @dblclick="startRename(tab)"
      :class="[
        'flex items-center gap-2 px-4 py-2 border rounded-t-lg text-sm font-medium transition-all cursor-pointer group',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-2xs -mb-px z-10'
          : 'bg-gray-50 border-gray-200 text-gray-500 hover:bg-gray-100 hover:text-gray-700'
      ]"
    >
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0',
          tab.id === activeTabId ? 'bg-blue-500' : 'bg-gray-300'
        ]"
      ></span>
      
      <template v-if="editingId === tab.id">
        <input
          ref="inputRef"
          v-model="editTitle"
          @blur="saveRename(tab.id)"
          @keyup.enter="saveRename(tab.id)"
          @click.stop
          class="px-1.5 py-0.5 border border-blue-500 rounded bg-white outline-none font-medium text-sm text-gray-900 w-24"
        />
      </template>
      <template v-else>
        <span>{{ tab.title }}</span>
      </template>

      <button
        v-if="tabs.length > 1"
        @click.stop="emit('close-tab', tab.id)"
        class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-gray-200/80 rounded transition cursor-pointer"
      >
        <X class="w-3.5 h-3.5 text-gray-400 hover:text-gray-600" />
      </button>
    </div>

    <button
      @click="emit('add-tab')"
      class="p-1.5 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition cursor-pointer"
      title="Add new device"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-ui-2-report.md`.
