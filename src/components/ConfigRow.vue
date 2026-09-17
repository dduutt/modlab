<script setup lang="ts">
export interface ModbusConfig {
  unitId: number;
  functionCode: string;
  startAddress: number;
  count: number;
  dataType: string;
  format: string;
  byteOrder: string;
  interval: number;
  raw: boolean;
}

const props = defineProps<{
  config: ModbusConfig;
  connected?: boolean;
  busy?: boolean;
  role?: 'Slave' | 'Master';
}>();

const emit = defineEmits<{
  (e: 'update:config', value: ModbusConfig): void;
}>();

function updateField<K extends keyof ModbusConfig>(key: K, value: ModbusConfig[K]) {
  if (typeof value === 'number' && !Number.isFinite(value)) return;
  const next = { ...props.config, [key]: value };
  next.unitId = Math.max(0, Math.min(255, Math.trunc(next.unitId)));
  next.startAddress = Math.max(0, Math.min(65535, Math.trunc(next.startAddress)));
  const maxCount = next.functionCode === '0x01' || next.functionCode === '0x02' ? 2000 : 125;
  next.count = Math.max(1, Math.min(maxCount, 65536 - next.startAddress, Math.trunc(next.count)));
  next.interval = Math.max(100, Math.min(60000, Math.trunc(next.interval)));
  emit('update:config', next);
}
</script>

<template>
  <fieldset :disabled="busy" class="flex flex-wrap shrink-0 items-center justify-between gap-2 px-6 py-2.5 bg-white border-b border-gray-100 text-xs select-none w-full min-w-0 min-h-[52px]">
    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">Unit ID</label>
      <input
        type="number"
        min="0"
        max="255"
        :value="config.unitId"
        :disabled="connected && role === 'Slave'"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          (connected && role === 'Slave')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.functionCode') }}</label>
      <select
        :value="config.functionCode"
        :disabled="connected"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border rounded-full outline-none font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer border-gray-200'
        ]"
      >
        <option value="0x01">0x01 Coils</option>
        <option value="0x02">0x02 Inputs</option>
        <option value="0x03">0x03 Holding</option>
        <option value="0x04">0x04 InputRegs</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.startAddress') }}</label>
      <input
        type="number"
        min="0"
        max="65535"
        :value="config.startAddress"
        :disabled="connected"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.count') }}</label>
      <input
        type="number"
        min="1"
        :max="config.functionCode === '0x01' || config.functionCode === '0x02' ? 2000 : 125"
        :value="config.count"
        :disabled="connected"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        :class="[
          'w-16 h-8 px-3 border rounded-full outline-none text-center font-medium text-xs shadow-2xs transition',
          connected
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-900 focus:border-blue-500 border-gray-200'
        ]"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.dataType') }}</label>
      <select
        :value="config.dataType"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border border-gray-200 rounded-full outline-none font-medium text-xs shadow-2xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="UInt32">UInt32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.format') }}</label>
      <select
        :value="config.format"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border border-gray-200 rounded-full outline-none font-medium text-xs shadow-2xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.byteOrder') }}</label>
      <select
        :value="config.byteOrder"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-8 px-3 border border-gray-200 rounded-full outline-none font-medium text-xs shadow-2xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-2">{{ $t('config.interval') }}</label>
      <input
        type="number"
        :value="config.interval"
        min="100"
        max="60000"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-20 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-start gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600 pl-1">Raw</label>
      <button
        type="button"
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-9 h-5 rounded-full p-0.5 transition-colors relative cursor-pointer my-1.5',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-4 h-4 rounded-full bg-white shadow-xs transition-transform',
            config.raw ? 'translate-x-4' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </fieldset>
</template>
