# Enhancement Task 1 Brief: Settings Modal Component (`SettingsModal.vue`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`
- Styling: TailwindCSS

## Instructions
Create `src/components/SettingsModal.vue` allowing full configuration of:
- Mode / Role (Slave Server vs Master Client)
- Protocol (Modbus TCP vs Modbus RTU Serial)
- IP Address & Port (for TCP)
- Serial Port, Baud Rate, Data Bits, Stop Bits, Parity (for RTU)

Code for `src/components/SettingsModal.vue`:
```vue
<script setup lang="ts">
import { ref, watch } from 'vue';
import { Settings, X, Server, Cpu, Cable, Network } from 'lucide-vue-next';

export interface ConnectionConfig {
  role: 'Slave' | 'Master';
  protocol: 'TCP' | 'RTU';
  ip: string;
  port: number;
  serialPort: string;
  baudRate: number;
  dataBits: number;
  stopBits: number;
  parity: 'None' | 'Even' | 'Odd';
}

const props = defineProps<{
  show: boolean;
  config: ConnectionConfig;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', config: ConnectionConfig): void;
}>();

const form = ref<ConnectionConfig>({ ...props.config });

watch(() => props.config, (newVal) => {
  form.value = { ...newVal };
}, { deep: true });

function handleSave() {
  emit('save', { ...form.value });
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center gap-2">
          <Settings class="w-5 h-5 text-blue-600" />
          <h3 class="font-semibold text-gray-900 text-base">Connection Settings</h3>
        </div>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="p-6 space-y-4 text-sm">
        <!-- Mode Switcher -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Mode / Role</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.role = 'Slave'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition cursor-pointer', form.role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Server class="w-4 h-4" /> Slave (Server)
            </button>
            <button
              type="button"
              @click="form.role = 'Master'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition cursor-pointer', form.role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cpu class="w-4 h-4" /> Master (Client)
            </button>
          </div>
        </div>

        <!-- Protocol Switcher -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">Protocol</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.protocol = 'TCP'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition cursor-pointer', form.protocol === 'TCP' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Network class="w-4 h-4" /> Modbus TCP
            </button>
            <button
              type="button"
              @click="form.protocol = 'RTU'"
              :class="['flex items-center justify-center gap-2 py-1.5 rounded-lg font-medium transition cursor-pointer', form.protocol === 'RTU' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              <Cable class="w-4 h-4" /> Modbus RTU (Serial)
            </button>
          </div>
        </div>

        <!-- TCP Fields -->
        <template v-if="form.protocol === 'TCP'">
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2">
              <label class="block text-xs font-semibold text-gray-500 mb-1">IP Address</label>
              <input v-model="form.ip" type="text" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-mono text-sm" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Port</label>
              <input v-model.number="form.port" type="number" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 font-mono text-sm text-center" />
            </div>
          </div>
        </template>

        <!-- RTU Fields -->
        <template v-else>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Serial Port</label>
              <select v-model="form.serialPort" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white font-mono text-sm cursor-pointer">
                <option value="COM1">COM1</option>
                <option value="COM2">COM2</option>
                <option value="COM3">COM3</option>
                <option value="/dev/ttyUSB0">/dev/ttyUSB0</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Baud Rate</label>
              <select v-model.number="form.baudRate" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white font-mono text-sm cursor-pointer">
                <option :value="4800">4800</option>
                <option :value="9600">9600</option>
                <option :value="19200">19200</option>
                <option :value="38400">38400</option>
                <option :value="115200">115200</option>
              </select>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Data Bits</label>
              <select v-model.number="form.dataBits" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center cursor-pointer">
                <option :value="7">7</option>
                <option :value="8">8</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Stop Bits</label>
              <select v-model.number="form.stopBits" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center cursor-pointer">
                <option :value="1">1</option>
                <option :value="2">2</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">Parity</label>
              <select v-model="form.parity" class="w-full px-3 py-2 border border-gray-200 rounded-xl outline-none focus:border-blue-500 bg-white text-center cursor-pointer">
                <option value="None">None</option>
                <option value="Even">Even</option>
                <option value="Odd">Odd</option>
              </select>
            </div>
          </div>
        </template>
      </div>

      <div class="flex items-center justify-end gap-3 px-6 py-4 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-2 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">Cancel</button>
        <button @click="handleSave" class="px-5 py-2 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer">Save Changes</button>
      </div>
    </div>
  </div>
</template>
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/enhancement-task-1-report.md`.
