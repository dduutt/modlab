# Task 1 Brief: Device Name Uniqueness Validation (`NewDeviceModal.vue` & `TabBar.vue` & `App.vue`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)

## Instructions
1. Update `src/components/NewDeviceModal.vue` to accept `existingNames: string[]` prop and validate device name uniqueness. Show red error text if input name matches any existing tab title.
2. Update `src/components/TabBar.vue` to accept `existingNames: string[]` or validate inline rename uniqueness against other tab titles.
3. Update `src/App.vue` to pass `existingNames` to `NewDeviceModal` and `TabBar`.

Code for `src/components/NewDeviceModal.vue`:
```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { Plus, X, Server, Cpu, AlertCircle } from '@lucide/vue';

const props = defineProps<{
  show: boolean;
  existingNames: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', deviceName: string, role: 'Slave' | 'Master'): void;
}>();

const deviceName = ref<string>('Slave 2');
const role = ref<'Slave' | 'Master'>('Slave');
const errorMessage = ref<string>('');

const isDuplicate = computed(() => {
  const name = deviceName.value.trim();
  return props.existingNames.some(n => n.toLowerCase() === name.toLowerCase());
});

function handleCreate() {
  const name = deviceName.value.trim() || (role.value === 'Slave' ? 'Slave 2' : 'Master 1');
  if (isDuplicate.value) {
    errorMessage.value = `Device name "${name}" already exists. Please enter a unique name.`;
    return;
  }
  errorMessage.value = '';
  emit('create', name, role.value);
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden animate-in fade-in zoom-in-95 duration-150 select-none">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <Plus class="w-5 h-5 text-blue-600" />
          <h3 class="font-semibold text-gray-900 text-base">Add New Device</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 space-y-4 text-sm">
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Device Name</label>
          <input
            v-model="deviceName"
            @input="errorMessage = ''"
            type="text"
            placeholder="e.g. Slave 2"
            class="w-full px-3.5 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-medium text-sm"
          />
          <div v-if="errorMessage" class="flex items-center gap-1 mt-1.5 text-xs text-red-600">
            <AlertCircle class="w-3.5 h-3.5 shrink-0" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Device Type (Role)</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="role = 'Slave'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Server class="w-4 h-4" /> Slave
            </button>
            <button
              type="button"
              @click="role = 'Master'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cpu class="w-4 h-4" /> Master
            </button>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="handleCreate" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Create Device</button>
      </div>
    </div>
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-multi-1-report.md`.
