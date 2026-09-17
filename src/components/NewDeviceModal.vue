<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { X, AlertCircle } from '@lucide/vue';

const props = defineProps<{
  show: boolean;
  existingNames: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', deviceName: string, role: 'Slave' | 'Master'): void;
}>();

const deviceName = ref<string>('');
const role = ref<'Slave' | 'Master'>('Slave');
const errorMessage = ref<string>('');
const userHasEdited = ref<boolean>(false);

function getSuggestedName(selectedRole: 'Slave' | 'Master'): string {
  const prefix = selectedRole;
  let index = 1;
  while (props.existingNames.some(n => n.toLowerCase() === `${prefix.toLowerCase()} ${index}`)) {
    index++;
  }
  return `${prefix} ${index}`;
}

watch(
  () => props.show,
  (isShowing: boolean) => {
    if (isShowing) {
      role.value = 'Slave';
      userHasEdited.value = false;
      deviceName.value = getSuggestedName('Slave');
      errorMessage.value = '';
    }
  },
  { immediate: true }
);

function selectRole(newRole: 'Slave' | 'Master') {
  role.value = newRole;
  if (!userHasEdited.value) {
    deviceName.value = getSuggestedName(newRole);
  }
  errorMessage.value = '';
}

function handleInput() {
  userHasEdited.value = true;
  errorMessage.value = '';
}

const isDuplicate = computed(() => {
  const name = deviceName.value.trim();
  return props.existingNames.some(n => n.toLowerCase() === name.toLowerCase());
});

function handleCreate() {
  const name = deviceName.value.trim() || getSuggestedName(role.value);
  if (isDuplicate.value) {
    errorMessage.value = 'modal.duplicateName';
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
          <h3 class="font-semibold text-gray-900 text-base">{{ $t('modal.addDevice') }}</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="p-5 space-y-4 text-sm">
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">{{ $t('modal.deviceName') }}</label>
          <input
            v-model="deviceName"
            @input="handleInput"
            type="text"
            :placeholder="$t('modal.e_g_slave2')"
            class="w-full px-3.5 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-medium text-sm"
          />
          <div v-if="errorMessage" class="flex items-center gap-1 mt-1.5 text-xs text-red-600">
            <AlertCircle class="w-3.5 h-3.5 shrink-0" />
            <span>{{ $t(errorMessage) }}</span>
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">{{ $t('modal.deviceType') }}</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="selectRole('Slave')"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              {{ $t('modal.slave') }}
            </button>
            <button
              type="button"
              @click="selectRole('Master')"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              {{ $t('modal.master') }}
            </button>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">{{ $t('common.cancel') }}</button>
        <button @click="handleCreate" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer">{{ $t('common.add') }}</button>
      </div>
    </div>
  </div>
</template>
