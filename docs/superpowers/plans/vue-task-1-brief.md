# Task 1 Brief: TabBar Component for Vue 3

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS / Vanilla CSS

## Instructions
Create `src/components/TabBar.vue` with tab list, tab selection, active indicator dot, close button, and add tab button.

Code for `src/components/TabBar.vue`:
```vue
<script setup lang="ts">
import { Plus, X } from 'lucide-vue-next';

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
}>();
</script>

<template>
  <div class="flex items-center bg-white px-4 pt-2 border-b border-gray-200 gap-2 select-none">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      @click="emit('select-tab', tab.id)"
      :class="[
        'flex items-center gap-2 px-4 py-2 border rounded-t-lg text-sm font-medium transition-all cursor-pointer group',
        tab.id === activeTabId
          ? 'bg-white border-gray-200 border-b-white text-gray-900 shadow-sm -mb-px z-10'
          : 'bg-gray-50 border-gray-200 text-gray-500 hover:bg-gray-100 hover:text-gray-700'
      ]"
    >
      <span
        :class="[
          'w-2 h-2 rounded-full',
          tab.id === activeTabId ? 'bg-blue-500' : 'bg-gray-300'
        ]"
      ></span>
      <span>{{ tab.title }}</span>
      <button
        v-if="tabs.length > 1"
        @click.stop="emit('close-tab', tab.id)"
        class="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-gray-200 rounded transition"
      >
        <X class="w-3.5 h-3.5 text-gray-400 hover:text-gray-600" />
      </button>
    </div>

    <button
      @click="emit('add-tab')"
      class="p-1.5 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition"
      title="Add new session"
    >
      <Plus class="w-4 h-4" />
    </button>
  </div>
</template>
```

Write full report to `file:///F:/pro/modlab/docs/superpowers/plans/vue-task-1-report.md`.
