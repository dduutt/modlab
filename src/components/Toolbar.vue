<script setup lang="ts">
import { Play, Square, Settings, Shuffle, RefreshCw, Repeat, Loader2 } from '@lucide/vue';

defineProps<{
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  connected: boolean;
  loading?: boolean;
  functionCode?: string;
  autoIncrement?: boolean;
  isPolling?: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-connect'): void;
  (e: 'open-settings'): void;
  (e: 'toggle-auto-increment'): void;
  (e: 'fill-random'): void;
  (e: 'toggle-poll'): void;
  (e: 'read-once'): void;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center justify-between px-6 py-2.5 bg-white border-b border-gray-200 select-none text-xs min-h-[46px] gap-2">
    <!-- Left: Status & Connection Info -->
    <div class="flex items-center gap-3">
      <span class="font-semibold text-gray-900 text-sm">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-xs text-gray-700 font-medium">{{ protocol }} {{ ip }}:{{ port }}</span>

      <!-- Connect / Listen Button (Fixed Label, Active Red State) -->
      <button
        @click="emit('toggle-connect')"
        :disabled="loading"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition cursor-pointer',
          loading ? 'opacity-75 cursor-wait' : '',
          connected
            ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100 shadow-2xs font-semibold'
            : 'bg-white text-gray-900 border-gray-300 hover:bg-gray-50'
        ]"
      >
        <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin text-gray-600" />
        <component v-else :is="connected ? Square : Play" class="w-3.5 h-3.5" />
        {{ role === 'Slave' ? 'Listen' : 'Connect' }}
      </button>

      <!-- Settings Button -->
      <button
        @click="!connected && emit('open-settings')"
        :disabled="connected"
        :title="connected ? 'Disconnect to change connection settings' : 'Connection Settings'"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
          connected
            ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50 cursor-pointer'
        ]"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        Settings
      </button>
    </div>

    <!-- Right: Mode-Specific Controls -->
    <div class="flex items-center gap-2.5">
      <!-- SLAVE MODE CONTROLS -->
      <template v-if="role === 'Slave'">
        <!-- Auto Increment Toggle (Fixed Label, Active Emerald State) -->
        <button
          @click="emit('toggle-auto-increment')"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition cursor-pointer',
            autoIncrement
              ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold'
              : 'bg-gray-100 text-gray-700 border-gray-200 hover:bg-gray-200'
          ]"
          title="Auto Increment values by 1 every Interval"
        >
          <Repeat class="w-3.5 h-3.5 text-gray-500" />
          Auto Increment
        </button>

        <!-- Random Action Button -->
        <button
          @click="emit('fill-random')"
          class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer border border-gray-200"
          title="Generate random values once"
        >
          <Shuffle class="w-3.5 h-3.5 text-gray-500" />
          Random
        </button>
      </template>

      <!-- MASTER MODE CONTROLS -->
      <template v-else>
        <!-- Poll Toggle (Fixed Label, Active Emerald State) -->
        <button
          @click="emit('toggle-poll')"
          :disabled="!connected"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : isPolling
                ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold cursor-pointer'
                : 'bg-gray-100 text-gray-700 border-gray-200 hover:bg-gray-200 cursor-pointer'
          ]"
          title="Poll slave continuously every Interval"
        >
          <RefreshCw :class="['w-3.5 h-3.5 text-gray-500', isPolling ? 'animate-spin' : '']" />
          Poll
        </button>

        <!-- Read Action Button -->
        <button
          @click="connected && emit('read-once')"
          :disabled="!connected"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200 border-gray-200 cursor-pointer'
          ]"
          title="Read values from slave once"
        >
          Read
        </button>
      </template>
    </div>
  </div>
</template>
