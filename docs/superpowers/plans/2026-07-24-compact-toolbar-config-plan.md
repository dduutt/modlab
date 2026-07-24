# Toolbar & ConfigRow Compact Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce padding, fonts, and control dimensions in Toolbar and ConfigRow to prevent overflow and preserve max vertical space for DataGrid.

**Architecture:** Update Vue 3 Tailwind utility classes in `Toolbar.vue` and `ConfigRow.vue` for tight padding, small icons/text, and single-row horizontal scrolling on narrow screens.

**Tech Stack:** Vue 3, TypeScript, Tailwind CSS, Lucide Icons.

## Global Constraints
- Keep all existing emit events, props, and bindings intact.
- Maintain responsive auto-fit behavior.

---

### Task 1: Compact `Toolbar.vue` Layout

**Files:**
- Modify: `src/components/Toolbar.vue`

**Interfaces:**
- Consumes: Existing props (`role`, `protocol`, `ip`, `port`, `connected`, `loading`, `functionCode`)
- Produces: Updated compact `Toolbar.vue` template styling

- [ ] **Step 1: Update outer container padding & font sizes in Toolbar.vue**

Replace lines 40-143 in `src/components/Toolbar.vue`:
```vue
<template>
  <div class="flex flex-wrap items-center justify-between px-3 py-1.5 bg-white border-b border-gray-200 select-none text-xs min-h-[38px] gap-2">
    <div class="flex items-center gap-2.5 flex-wrap">
      <span class="font-semibold text-gray-900 text-xs">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-xs text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <button
        @click="emit('toggle-connect')"
        :disabled="loading"
        :class="[
          'flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium border transition cursor-pointer',
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
        class="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-white text-gray-700 border border-gray-200 hover:bg-gray-50 transition cursor-pointer"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        Settings
      </button>

      <button
        @click="emit('open-traffic-log')"
        class="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-slate-900 text-slate-100 hover:bg-slate-800 transition cursor-pointer shadow-2xs"
        title="Open Traffic Log Window"
      >
        <Activity class="w-3.5 h-3.5 text-sky-400" />
        Traffic Log
      </button>
    </div>

    <div class="flex items-center gap-2 flex-wrap">
      <button
        @click="!isReadOnly && emit('fill-random')"
        :disabled="isReadOnly"
        :title="isReadOnly ? 'Read-only function code; data cannot be written' : 'Fill Random Values'"
        :class="[
          'flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition',
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
          'flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition',
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
        class="flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer"
      >
        <Download class="w-3.5 h-3.5 text-gray-500" />
        Export
      </button>

      <button
        @click="!isReadOnly && fileInput?.click()"
        :disabled="isReadOnly"
        :title="isReadOnly ? 'Read-only function code; data cannot be imported' : 'Import Registers from CSV'"
        :class="[
          'flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition',
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

- [ ] **Step 2: Check build/type errors**

Run: `npx vue-tsc --noEmit` or `npm run build`
Expected: Build succeeds with 0 errors.

---

### Task 2: Compact `ConfigRow.vue` Layout

**Files:**
- Modify: `src/components/ConfigRow.vue`

**Interfaces:**
- Consumes: Existing `ModbusConfig` interface & `v-model:config`
- Produces: Compact single-row auto-scrolling `ConfigRow.vue`

- [ ] **Step 1: Update ConfigRow template to single-row tight form items**

Replace lines 27-148 in `src/components/ConfigRow.vue`:
```vue
<template>
  <div class="flex items-center gap-3 px-3 py-1.5 bg-white border-b border-gray-100 text-xs overflow-x-auto whitespace-nowrap min-h-[36px] select-none">
    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Unit ID</label>
      <input
        type="number"
        :value="config.unitId"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        class="w-14 h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Function</label>
      <select
        :value="config.functionCode"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        class="h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="0x01">Read Coils (0x01)</option>
        <option value="0x02">Read Discrete Inputs (0x02)</option>
        <option value="0x03">Holding (0x03)</option>
        <option value="0x04">Read Input Registers (0x04)</option>
      </select>
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Count</label>
      <input
        type="number"
        :value="config.count"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Data Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Format</label>
      <select
        :value="config.format"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-7 px-2 border border-gray-200 rounded-md outline-none font-medium text-xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
      </select>
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Byte Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex items-center gap-1.5 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Interval</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-20 h-7 px-2 border border-gray-200 rounded-md outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1.5 shrink-0 ml-auto">
      <label class="text-[10px] font-semibold text-gray-500 uppercase">Raw</label>
      <button
        type="button"
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-8 h-4 rounded-full p-0.5 transition-colors relative cursor-pointer',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-3 h-3 rounded-full bg-white shadow-sm transition-transform',
            config.raw ? 'translate-x-4' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Check build/type errors & verify functionality**

Run: `npx vue-tsc --noEmit` or `npm run build`
Expected: Build succeeds cleanly.
