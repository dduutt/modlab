# Task 3 Brief: Configuration Panel Component (`ConfigRow.vue`) for Vue 3

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Styling: TailwindCSS / Vanilla CSS

## Instructions
Create `src/components/ConfigRow.vue` to allow configuring Unit ID, Function code, Start address, Count, Data Type, Format, Byte Order, Interval, and Raw toggle.

Code for `src/components/ConfigRow.vue`:
```vue
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
}>();

const emit = defineEmits<{
  (e: 'update:config', value: ModbusConfig): void;
}>();

function updateField<K extends keyof ModbusConfig>(key: K, value: ModbusConfig[K]) {
  emit('update:config', { ...props.config, [key]: value });
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-4 px-4 py-3 bg-white border-b border-gray-200 text-sm">
    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Unit ID</label>
      <input
        type="number"
        :value="config.unitId"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        class="w-16 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Function</label>
      <select
        :value="config.functionCode"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="0x01">Read Coils (0x01)</option>
        <option value="0x02">Read Discrete Inputs (0x02)</option>
        <option value="0x03">Holding (0x03)</option>
        <option value="0x04">Read Input Registers (0x04)</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        class="w-20 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Count</label>
      <input
        type="number"
        :value="config.count"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        class="w-20 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Data Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Format</label>
      <select
        :value="config.format"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
        <option value="Bin">Bin</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Byte Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="px-3 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 bg-white"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-xs font-medium text-gray-500">Interval</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-24 px-2.5 py-1.5 border border-gray-200 rounded-lg outline-none focus:border-blue-500 text-center"
      />
    </div>

    <div class="flex flex-col gap-1 items-center ml-auto">
      <label class="text-xs font-medium text-gray-500">Raw</label>
      <button
        type="button"
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-9 h-5 rounded-full p-0.5 transition-colors relative',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-4 h-4 rounded-full bg-white shadow-sm transition-transform',
            config.raw ? 'translate-x-4' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </div>
</template>
```

Write full report to `file:///F:/pro/modlab/docs/superpowers/plans/vue-task-3-report.md`.
