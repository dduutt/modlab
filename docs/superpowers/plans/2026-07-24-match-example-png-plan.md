# Match 示例.png Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replicate the design, rounded pill controls, and exact labels from `示例.png` in `ConfigRow.vue` and `Toolbar.vue`.

**Architecture:** Update `ConfigRow.vue` and `Toolbar.vue` templates to match `示例.png`.

**Tech Stack:** Vue 3, Tailwind CSS, TypeScript, pnpm.

## Global Constraints
- Must use `pnpm` for build verification.
- Preserve all emits and bindings.

---

### Task 1: Update `ConfigRow.vue` to match `示例.png`

**Files:**
- Modify: `src/components/ConfigRow.vue`

- [ ] **Step 1: Replace template in ConfigRow.vue**

Replace `ConfigRow.vue` template with:
```vue
<template>
  <div class="flex items-center justify-between gap-2 px-6 py-2.5 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[52px]">
    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Unit ID</label>
      <input
        type="number"
        :value="config.unitId"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Function</label>
      <select
        :value="config.functionCode"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        class="h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 bg-white font-medium text-xs shadow-2xs cursor-pointer"
      >
        <option value="0x01">Read Coils (0x01)</option>
        <option value="0x02">Read Discrete Inputs (0x02)</option>
        <option value="0x03">Holding (0x03)</option>
        <option value="0x04">Read Input Registers (0x04)</option>
      </select>
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Count</label>
      <input
        type="number"
        :value="config.count"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Data Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 bg-white font-medium text-xs shadow-2xs cursor-pointer"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Format</label>
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

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Byte Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 bg-white font-medium text-xs shadow-2xs cursor-pointer"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Interval</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-20 h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs"
      />
    </div>

    <div class="flex flex-col items-center gap-1 shrink-0">
      <label class="text-xs font-semibold text-gray-600">Raw</label>
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
  </div>
</template>
```

---

### Task 2: Update `Toolbar.vue` to match `示例.png`

**Files:**
- Modify: `src/components/Toolbar.vue`

- [ ] **Step 1: Replace template in Toolbar.vue**

Replace `Toolbar.vue` template with:
```vue
<template>
  <div class="flex flex-wrap items-center justify-between px-6 py-2.5 bg-white border-b border-gray-200 select-none text-xs min-h-[46px] gap-2">
    <div class="flex items-center gap-3">
      <span class="font-semibold text-gray-900 text-sm">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-xs text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <button
        @click="emit('toggle-connect')"
        :disabled="loading"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition cursor-pointer',
          loading ? 'opacity-75 cursor-wait' : '',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100'
            : 'bg-white text-gray-900 border-gray-300 hover:bg-gray-50'
        ]"
      >
        <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin text-gray-600" />
        <component v-else :is="connected ? Square : Play" class="w-3.5 h-3.5" />
        {{ loading ? 'Connecting...' : connected ? 'Disconnect' : 'Connect' }}
      </button>

      <button
        @click="emit('open-settings')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium bg-white text-gray-700 border border-gray-200 hover:bg-gray-50 transition cursor-pointer"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        Settings
      </button>

      <button
        @click="emit('open-traffic-log')"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium bg-slate-900 text-slate-100 hover:bg-slate-800 transition cursor-pointer shadow-2xs"
        title="Open Traffic Log Window"
      >
        <Activity class="w-3.5 h-3.5 text-sky-400" />
        Traffic Log
      </button>
    </div>

    <div class="flex items-center gap-2.5">
      <button
        @click="!isReadOnly && emit('fill-random')"
        :disabled="isReadOnly"
        :title="isReadOnly ? 'Read-only function code; data cannot be written' : 'Fill Random Values'"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium transition',
          isReadOnly
            ? 'bg-gray-100 text-gray-400 border border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-gray-100 text-gray-700 hover:bg-gray-200 cursor-pointer'
        ]"
      >
        <Shuffle class="w-3.5 h-3.5 text-gray-500" />
        Random
      </button>

      <button
        @click="!isReadOnly && emit('fill-increment')"
        :disabled="isReadOnly"
        :title="isReadOnly ? 'Read-only function code; data cannot be written' : 'Fill Incrementing Values'"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium transition',
          isReadOnly
            ? 'bg-gray-100 text-gray-400 border border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-gray-100 text-gray-700 hover:bg-gray-200 cursor-pointer'
        ]"
      >
        <TrendingUp class="w-3.5 h-3.5 text-gray-500" />
        Increment
      </button>

      <button
        @click="emit('export-csv')"
        title="Export Registers to CSV"
        class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer"
      >
        <Download class="w-3.5 h-3.5 text-gray-500" />
        Export
      </button>

      <button
        @click="!isReadOnly && fileInput?.click()"
        :disabled="isReadOnly"
        :title="isReadOnly ? 'Read-only function code; data cannot be imported' : 'Import Registers from CSV'"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium transition',
          isReadOnly
            ? 'bg-gray-100 text-gray-400 border border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-gray-100 text-gray-700 hover:bg-gray-200 cursor-pointer'
        ]"
      >
        <Upload class="w-3.5 h-3.5 text-gray-500" />
        Import
      </button>
      <input
        ref="fileInput"
        type="file"
        accept=".csv"
        class="hidden"
        @change="handleFileChange"
      />
    </div>
  </div>
</template>
```

- [ ] **Step 2: Run `pnpm run build` verification**
