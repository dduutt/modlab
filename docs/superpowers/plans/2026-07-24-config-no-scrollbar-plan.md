# ConfigRow No-Scrollbar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove horizontal scrollbar from `ConfigRow.vue` by tightening labels, widths, and options text.

**Architecture:** Update `ConfigRow.vue` template Tailwind classes.

**Tech Stack:** Vue 3, TypeScript, Tailwind CSS, pnpm.

## Global Constraints
- Must use `pnpm` for all commands.
- Keep all props (`config`) and emits (`update:config`) intact.

---

### Task 1: Update `ConfigRow.vue` for No-Scrollbar Single-Row Layout

**Files:**
- Modify: `src/components/ConfigRow.vue`

- [ ] **Step 1: Replace template styling in ConfigRow.vue**

Replace lines 27-148 in `src/components/ConfigRow.vue`:
```vue
<template>
  <div class="flex items-center justify-between gap-1.5 px-2 py-1 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[34px]">
    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">ID</label>
      <input
        type="number"
        :value="config.unitId"
        @input="updateField('unitId', Number(($event.target as HTMLInputElement).value))"
        class="w-11 h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Func</label>
      <select
        :value="config.functionCode"
        @change="updateField('functionCode', ($event.target as HTMLSelectElement).value)"
        class="h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="0x01">0x01 Coils</option>
        <option value="0x02">0x02 Inputs</option>
        <option value="0x03">0x03 Holding</option>
        <option value="0x04">0x04 InputRegs</option>
      </select>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Start</label>
      <input
        type="number"
        :value="config.startAddress"
        @input="updateField('startAddress', Number(($event.target as HTMLInputElement).value))"
        class="w-14 h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Count</label>
      <input
        type="number"
        :value="config.count"
        @input="updateField('count', Number(($event.target as HTMLInputElement).value))"
        class="w-14 h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Type</label>
      <select
        :value="config.dataType"
        @change="updateField('dataType', ($event.target as HTMLSelectElement).value)"
        class="h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="Int16">Int16</option>
        <option value="UInt16">UInt16</option>
        <option value="Int32">Int32</option>
        <option value="Float32">Float32</option>
      </select>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Fmt</label>
      <select
        :value="config.format"
        :disabled="config.functionCode === '0x01' || config.functionCode === '0x02'"
        @change="updateField('format', ($event.target as HTMLSelectElement).value)"
        :class="[
          'h-6 px-1 border border-gray-200 rounded outline-none font-medium text-xs transition',
          (config.functionCode === '0x01' || config.functionCode === '0x02')
            ? 'bg-gray-100 text-gray-400 cursor-not-allowed border-gray-200'
            : 'bg-white text-gray-800 focus:border-blue-500 cursor-pointer'
        ]"
      >
        <option value="Dec">Dec</option>
        <option value="Hex">Hex</option>
      </select>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Order</label>
      <select
        :value="config.byteOrder"
        @change="updateField('byteOrder', ($event.target as HTMLSelectElement).value)"
        class="h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 bg-white font-medium text-xs cursor-pointer"
      >
        <option value="ABCD">ABCD</option>
        <option value="CDAB">CDAB</option>
        <option value="BADC">BADC</option>
        <option value="DCBA">DCBA</option>
      </select>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Intv</label>
      <input
        type="number"
        :value="config.interval"
        @input="updateField('interval', Number(($event.target as HTMLInputElement).value))"
        class="w-16 h-6 px-1 border border-gray-200 rounded outline-none focus:border-blue-500 text-center font-medium bg-white text-xs"
      />
    </div>

    <div class="flex items-center gap-1 shrink-0">
      <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">Raw</label>
      <button
        type="button"
        @click="updateField('raw', !config.raw)"
        :class="[
          'w-7 h-3.5 rounded-full p-0.5 transition-colors relative cursor-pointer',
          config.raw ? 'bg-blue-600' : 'bg-gray-200'
        ]"
      >
        <div
          :class="[
            'w-2.5 h-2.5 rounded-full bg-white shadow-xs transition-transform',
            config.raw ? 'translate-x-3' : 'translate-x-0'
          ]"
        ></div>
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Run verification with pnpm**

Run: `pnpm run build`
Expected: `vue-tsc --noEmit` clean, build success.
