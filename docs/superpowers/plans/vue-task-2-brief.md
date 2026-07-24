# Task 2 Brief: Toolbar Component (`Toolbar.vue`) for Vue 3

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS / Vanilla CSS

## Instructions
Create `src/components/Toolbar.vue` for the Modbus connection status, role display, IP/Port info, Connect/Disconnect button, Settings button, and utility buttons (Random, Increment).

Code for `src/components/Toolbar.vue`:
```vue
<script setup lang="ts">
import { Settings, Play, Square, Shuffle, TrendingUp } from 'lucide-vue-next';

defineProps<{
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connect'): void;
  (e: 'open-settings'): void;
  (e: 'fill-random'): void;
  (e: 'fill-increment'): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between px-4 py-3 bg-white border-b border-gray-200">
    <div class="flex items-center gap-4">
      <span class="font-semibold text-gray-900 text-base">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-sm text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <button
        @click="emit('toggle-connect')"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium border transition',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100'
            : 'bg-white text-gray-900 border-gray-300 hover:bg-gray-50'
        ]"
      >
        <component :is="connected ? Square : Play" class="w-4 h-4" />
        {{ connected ? 'Disconnect' : 'Connect' }}
      </button>

      <button
        @click="emit('open-settings')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-white text-gray-700 border border-gray-200 hover:bg-gray-50 transition"
      >
        <Settings class="w-4 h-4 text-gray-500" />
        Settings
      </button>
    </div>

    <div class="flex items-center gap-3">
      <button
        @click="emit('fill-random')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition"
      >
        <Shuffle class="w-3.5 h-3.5 text-gray-500" />
        Random
      </button>
      <button
        @click="emit('fill-increment')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-sm font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition"
      >
        <TrendingUp class="w-3.5 h-3.5 text-gray-500" />
        Increment
      </button>
    </div>
  </div>
</template>
```

Write full report to `file:///F:/pro/modlab/docs/superpowers/plans/vue-task-2-report.md`.
