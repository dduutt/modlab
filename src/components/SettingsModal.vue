<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue';
import { X } from '@lucide/vue';
import { ModbusService } from '../services/modbusService';
import { connectionError } from '../utils/configValidation';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

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
  mode?: 'create' | 'edit';
  existingNames?: string[];
  deviceName?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', newConfig: ConnectionConfig, deviceName?: string): void;
  (e: 'create', deviceName: string, config: ConnectionConfig): void;
}>();

const form = ref<ConnectionConfig>({ ...props.config });
const deviceName = ref<string>(props.mode === 'create' ? '' : (props.deviceName || 'Device'));
const userHasEdited = ref<boolean>(false);
const serialPorts = ref<string[]>([]);
const portsLoading = ref(false);
const portsRefreshing = ref(false);
const portsError = ref('');
let portRequestId = 0;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;
const SERIAL_REFRESH_INTERVAL_MS = 1000;

const submitError = ref<string>('');
const invalidField = ref<string>('');

const nameInputRef = ref<HTMLInputElement | null>(null);
const ipInputRef = ref<HTMLInputElement | null>(null);
const portInputRef = ref<HTMLInputElement | null>(null);
const serialSelectRef = ref<HTMLSelectElement | null>(null);
const timeoutInputRef = ref<HTMLInputElement | null>(null);
const retriesInputRef = ref<HTMLInputElement | null>(null);

function focusField(field: string) {
  setTimeout(() => {
    if (field === 'name') nameInputRef.value?.focus();
    else if (field === 'ip') ipInputRef.value?.focus();
    else if (field === 'port') portInputRef.value?.focus();
    else if (field === 'serialPort') serialSelectRef.value?.focus();
    else if (field === 'timeoutMs') timeoutInputRef.value?.focus();
    else if (field === 'retries') retriesInputRef.value?.focus();
  }, 0);
}

function selectRole(newRole: 'Slave' | 'Master') {
  if (props.mode === 'edit') return;
  form.value.role = newRole;
}

function handleNameInput() {
  userHasEdited.value = true;
}

function stopSerialRefresh() {
  ++portRequestId;
  clearTimeout(refreshTimer);
  refreshTimer = undefined;
  portsLoading.value = false;
  portsRefreshing.value = false;
}

const isDuplicateName = computed(() => {
  const trimmed = deviceName.value.trim().toLowerCase();
  if (!trimmed) return false;
  const names = props.existingNames || [];
  if (props.mode === 'create') {
    return names.some(n => n.toLowerCase() === trimmed);
  }
  const current = (props.deviceName || '').toLowerCase();
  return names.some(n => n.toLowerCase() === trimmed && n.toLowerCase() !== current);
});

function validateAll(): { error: string; field: string } | null {
  const trimmedName = deviceName.value.trim();
  if (!trimmedName) {
    return { error: 'modal.nameRequired', field: 'name' };
  }
  if (isDuplicateName.value) {
    return { error: 'modal.duplicateName', field: 'name' };
  }

  if (form.value.protocol === 'TCP') {
    const ip = form.value.ip.trim();
    const ipv4 = /^\d{1,3}(\.\d{1,3}){3}$/.test(ip) && ip.split('.').every(part => Number(part) <= 255 && (part === '0' || !part.startsWith('0')));
    let ipv6 = false;
    if (ip.startsWith('[') && ip.endsWith(']')) {
      try { ipv6 = new URL(`http://${ip}`).hostname.startsWith('['); } catch { /* invalid */ }
    }
    if (!ipv4 && !ipv6) {
      return { error: 'modal.invalidIp', field: 'ip' };
    }
    if (!Number.isInteger(form.value.port) || form.value.port < 1 || form.value.port > 65535) {
      return { error: 'modal.invalidPort', field: 'port' };
    }
  } else {
    if (portsLoading.value || !!portsError.value || !form.value.serialPort || !serialPorts.value.includes(form.value.serialPort)) {
      return { error: portsError.value ? 'modal.serialPortsError' : 'modal.selectSerialPort', field: 'serialPort' };
    }
  }

  if (!Number.isInteger(form.value.timeoutMs) || form.value.timeoutMs < 100 || form.value.timeoutMs > 10000) {
    return { error: 'modal.invalidTimeout', field: 'timeoutMs' };
  }
  if (!Number.isInteger(form.value.retries) || form.value.retries < 0 || form.value.retries > 10) {
    return { error: 'modal.invalidRetries', field: 'retries' };
  }

  return null;
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
    if (show) {
      form.value = { ...config };
      userHasEdited.value = false;
      submitError.value = '';
      invalidField.value = '';
      if (props.mode === 'create') {
        form.value.role = form.value.role || 'Slave';
        form.value.port = form.value.port || 502;
        form.value.ip = form.value.ip || '127.0.0.1';
        deviceName.value = '';
      } else {
        deviceName.value = props.deviceName || 'Device';
      }
    }
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
  const err = validateAll();
  if (err) {
    submitError.value = err.error;
    invalidField.value = err.field;
    focusField(err.field);
    return;
  }
  submitError.value = '';
  invalidField.value = '';
  const name = deviceName.value.trim();
  const config = { ...form.value, ip: form.value.ip.trim() };
  emit('save', config, name);
  if (props.mode === 'create') {
    emit('create', name, config);
  }
  emit('close');
}
</script>

<template>
  <Dialog :open="show" @update:open="(val: boolean) => { if (!val) emit('close'); }">
    <DialogContent
      :aria-label="mode === 'create' ? $t('modal.addDevice') : $t('modal.connectionSettings')"
      :data-can-save="canSave"
      class="max-w-md sm:max-w-md max-h-[calc(100vh-2rem)] flex flex-col p-0 overflow-hidden bg-white border border-gray-100 shadow-2xl rounded-2xl select-none"
      :show-close-button="false"
    >
      <!-- Modal Header -->
      <DialogHeader class="flex flex-row shrink-0 items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <DialogTitle class="font-semibold text-gray-900 text-base">
          {{ mode === 'create' ? $t('modal.addDevice') : $t('modal.connectionSettings') }}
        </DialogTitle>
        <button type="button" @click="emit('close')" class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </DialogHeader>

        <!-- Modal Body -->
        <div class="flex-1 p-5 space-y-4 text-sm min-h-0 overflow-y-auto">
          <!-- Device Name & Role -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1.5">{{ $t('modal.deviceName') }}</label>
              <input
                ref="nameInputRef"
                v-model="deviceName"
                @input="handleNameInput"
                type="text"
                :class="[
                  'w-full h-9 px-3 py-1.5 border rounded-lg outline-none bg-white font-medium text-sm transition-colors',
                  invalidField === 'name' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              />
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1.5">{{ $t('modal.deviceType') }}</label>
              <div class="grid grid-cols-2 gap-1 bg-gray-100 p-1 rounded-lg h-9">
                <button
                  type="button"
                  @click="selectRole('Slave')"
                  :disabled="mode === 'edit'"
                  :class="[
                    'flex items-center justify-center rounded-md font-medium text-xs transition',
                    form.role === 'Slave' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900',
                    mode === 'edit' ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'
                  ]"
                >
                  {{ $t('modal.slave') }}
                </button>
                <button
                  type="button"
                  @click="selectRole('Master')"
                  :disabled="mode === 'edit'"
                  :class="[
                    'flex items-center justify-center rounded-md font-medium text-xs transition',
                    form.role === 'Master' ? 'bg-white text-blue-600 shadow-2xs' : 'text-gray-600 hover:text-gray-900',
                    mode === 'edit' ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'
                  ]"
                >
                  {{ $t('modal.master') }}
                </button>
              </div>
            </div>
          </div>

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
          <div v-if="form.protocol === 'TCP'" class="grid grid-cols-3 content-start gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
            <div class="col-span-2">
              <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.ipAddress') }}</label>
              <input
                ref="ipInputRef"
                v-model="form.ip"
                type="text"
                :class="[
                  'w-full h-9 px-3 py-1.5 border rounded-lg outline-none bg-white font-mono text-sm transition-colors',
                  invalidField === 'ip' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              />
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.port') }}</label>
              <input
                ref="portInputRef"
                v-model.number="form.port"
                type="number"
                :class="[
                  'w-full h-9 px-3 py-1.5 border rounded-lg outline-none bg-white font-mono text-sm transition-colors',
                  invalidField === 'port' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              />
            </div>
          </div>

          <!-- RTU Core Options -->
          <div v-else class="grid grid-cols-3 content-start gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
            <div class="col-span-2 min-w-0">
              <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.serialPort') }}</label>
              <select
                ref="serialSelectRef"
                v-model="form.serialPort"
                :disabled="portsLoading || serialPorts.length === 0"
                :class="[
                  'w-full h-9 min-w-0 px-3 py-1.5 border rounded-lg outline-none bg-white font-mono text-sm cursor-pointer disabled:opacity-60 transition-colors',
                  invalidField === 'serialPort' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              >
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
          </div>

          <!-- RTU Serial Options (Data Bits, Stop Bits, Parity) with smooth height transition -->
          <div
            class="grid transition-all duration-200 ease-in-out"
            :class="form.protocol === 'RTU' ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0 pointer-events-none'"
          >
            <div class="overflow-hidden">
              <div class="grid grid-cols-3 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
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
            </div>
          </div>

          <!-- Timeout & Retries -->
          <div class="grid grid-cols-2 gap-3 bg-gray-50 p-3 rounded-xl border border-gray-100">
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.timeout') }}</label>
              <input
                ref="timeoutInputRef"
                v-model.number="form.timeoutMs"
                type="number"
                min="100"
                max="10000"
                step="100"
                :class="[
                  'w-full h-9 px-3 py-1.5 border rounded-lg outline-none bg-white font-mono text-sm transition-colors',
                  invalidField === 'timeoutMs' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              />
            </div>
            <div>
              <label class="block text-xs font-semibold text-gray-500 mb-1">{{ $t('modal.retries') }}</label>
              <input
                ref="retriesInputRef"
                v-model.number="form.retries"
                type="number"
                min="0"
                max="10"
                :class="[
                  'w-full h-9 px-3 py-1.5 border rounded-lg outline-none bg-white font-mono text-sm transition-colors',
                  invalidField === 'retries' ? 'border-red-500 focus:border-red-500 ring-1 ring-red-500/20' : 'border-gray-200 focus:border-blue-500'
                ]"
              />
            </div>
          </div>
        </div>

        <!-- Validation Error Message -->
        <p v-if="submitError" role="alert" class="px-5 pb-2 text-xs text-red-600 font-medium animate-in fade-in duration-150">
          {{ $t(submitError) }}
        </p>

        <!-- Footer Buttons -->
        <DialogFooter class="flex shrink-0 items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
          <button type="button" @click="emit('close')" class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer">{{ $t('common.cancel') }}</button>
          <button type="button" @click="handleSave" class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer active:scale-98">
            {{ mode === 'create' ? $t('modal.createDevice') : $t('common.save') }}
          </button>
        </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
