<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue';
import { X } from '@lucide/vue';
import { ModbusService } from '../services/modbusService';
import { connectionError } from '../utils/configValidation';

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
  timeoutMs: number;
  retries: number;
}

const props = defineProps<{
  show: boolean;
  config: ConnectionConfig;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', newConfig: ConnectionConfig): void;
}>();

const form = ref<ConnectionConfig>({ ...props.config });
const serialPorts = ref<string[]>([]);
const portsLoading = ref(false);
const portsRefreshing = ref(false);
const portsError = ref('');
let portRequestId = 0;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;
const SERIAL_REFRESH_INTERVAL_MS = 1000;

function stopSerialRefresh() {
  ++portRequestId;
  clearTimeout(refreshTimer);
  refreshTimer = undefined;
  portsLoading.value = false;
  portsRefreshing.value = false;
}

const validationError = computed(() => connectionError(form.value));
const canSave = computed(() => !validationError.value && (form.value.protocol !== 'RTU' || (
  !portsLoading.value && !portsError.value && serialPorts.value.includes(form.value.serialPort)
)));

async function refreshSerialPorts(background = false) {
  if (!props.show || form.value.protocol !== 'RTU' || portsRefreshing.value) return;
  clearTimeout(refreshTimer);
  refreshTimer = undefined;
  const requestId = ++portRequestId;
  portsRefreshing.value = true;
  portsLoading.value = !background;
  if (!background) {
    portsError.value = '';
    serialPorts.value = [];
  }
  try {
    const ports = await ModbusService.listSerialPorts();
    if (requestId !== portRequestId) return;
    const detected = [...new Set(ports)].sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
    if (detected.length !== serialPorts.value.length || detected.some((port, index) => port !== serialPorts.value[index])) {
      serialPorts.value = detected;
    }
    portsError.value = '';
    if (!detected.includes(form.value.serialPort)) {
      // A hot-unplug must not silently select a different device.
      form.value.serialPort = background ? '' : detected[0] ?? '';
    }
  } catch (err) {
    if (requestId !== portRequestId) return;
    if (!background) form.value.serialPort = '';
    portsError.value = err instanceof Error ? err.message : String(err);
  } finally {
    if (requestId === portRequestId) {
      portsLoading.value = false;
      portsRefreshing.value = false;
      if (props.show && form.value.protocol === 'RTU') {
        refreshTimer = setTimeout(() => refreshSerialPorts(true), SERIAL_REFRESH_INTERVAL_MS);
      }
    }
  }
}

watch(
  [() => props.show, () => props.config],
  ([show, config]) => {
    if (show) form.value = { ...config };
  },
  { deep: true, immediate: true }
);

watch(
  () => [props.show, form.value.protocol] as const,
  ([show, protocol]) => {
    stopSerialRefresh();
    if (show && protocol === 'RTU') {
      void refreshSerialPorts();
    }
  },
  { immediate: true }
);

onUnmounted(stopSerialRefresh);


function handleSave() {
  if (!canSave.value) return;
  emit('save', { ...form.value, ip: form.value.ip.trim() });
  emit('close');
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div role="dialog" aria-modal="true" :aria-label="$t('modal.connectionSettings')" class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-md h-[520px] max-h-[calc(100vh-2rem)] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <div class="flex shrink-0 items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <h3 class="font-semibold text-gray-900 text-base">{{ $t('modal.connectionSettings') }}</h3>
        <button @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 p-5 space-y-4 text-sm min-h-0 overflow-y-auto">
        <!-- Protocol -->
        <div>
          <label class="block text-xs font-semibold text-gray-500 mb-1.5">{{ $t('modal.protocolType') }}</label>
          <div class="grid grid-cols-2 gap-2 bg-gray-100 p-1 rounded-xl">
            <button
              type="button"
              @click="form.protocol = 'TCP'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.protocol === 'TCP' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              Modbus TCP
            </button>
            <button
              type="button"
              @click="form.protocol = 'RTU'"
              :class="['flex items-center justify-center gap-2 py-2 rounded-lg font-medium transition cursor-pointer', form.protocol === 'RTU' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900']"
            >
              Modbus RTU
            </button>
          </div>
        </div>

        <!-- TCP Options -->
        <div v-if="form.protocol === 'TCP'" class="grid grid-cols-3 content-start min-h-[172px] gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div class="col-span-2">
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.ipAddress') }}</label>
            <input v-model="form.ip" type="text" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.port') }}</label>
            <input v-model.number="form.port" type="number" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
        </div>

        <!-- RTU Options -->
        <div v-else class="grid grid-cols-3 content-start min-h-[172px] gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div class="col-span-2 min-w-0">
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.serialPort') }}</label>
            <select v-model="form.serialPort" :disabled="portsLoading || serialPorts.length === 0" class="w-full h-9 min-w-0 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm cursor-pointer disabled:opacity-60">
              <option value="" disabled>{{ portsLoading ? $t('modal.scanningPorts') : $t('modal.selectSerialPort') }}</option>
              <option v-for="port in serialPorts" :key="port" :value="port">{{ port }}</option>
            </select>
            <p v-if="portsError" role="alert" class="mt-1 text-xs text-red-600" :title="portsError">{{ $t('modal.serialPortsError') }}</p>
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.baudRate') }}</label>
            <select v-model.number="form.baudRate" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option :value="9600">9600</option>
              <option :value="19200">19200</option>
              <option :value="38400">38400</option>
              <option :value="57600">57600</option>
              <option :value="115200">115200</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.dataBits') }}</label>
            <select v-model.number="form.dataBits" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option :value="7">7</option>
              <option :value="8">8</option>
            </select>
          </div>
          <div>
            <label for="serial-stop-bits" class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.stopBits') }}</label>
            <select id="serial-stop-bits" v-model.number="form.stopBits" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option :value="1">1</option>
              <option :value="2">2</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.parity') }}</label>
            <select v-model="form.parity" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm">
              <option value="None">{{ $t('modal.parityNone') }}</option>
              <option value="Even">{{ $t('modal.parityEven') }}</option>
              <option value="Odd">{{ $t('modal.parityOdd') }}</option>
            </select>
          </div>
        </div>

        <!-- Timeout & Retries -->
        <div class="grid grid-cols-2 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.timeout') }}</label>
            <input v-model.number="form.timeoutMs" type="number" min="100" max="10000" step="100" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
          <div>
            <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.retries') }}</label>
            <input v-model.number="form.retries" type="number" min="0" max="10" class="w-full h-9 px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white font-mono text-sm" />
          </div>
        </div>
      </div>

      <p v-if="validationError" role="alert" class="px-5 pb-3 text-xs text-red-600">{{ $t(validationError) }}</p>
      <div class="flex shrink-0 items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">{{ $t('common.cancel') }}</button>
        <button @click="handleSave" :disabled="!canSave" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed">{{ $t('common.save') }}</button>
      </div>
    </div>
  </div>
</template>
