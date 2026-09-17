<script setup lang="ts">
import TcpClients from './TcpClients.vue';
import { Play, Square, Settings, Shuffle, RefreshCw, Repeat, Loader2 } from '@lucide/vue';

defineProps<{
  sessionId: string;
  role: 'Slave' | 'Master';
  protocol: string;
  ip: string;
  port: number;
  serialPort?: string;
  baudRate?: number;
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
    <div class="flex flex-wrap items-center gap-3">
      <span class="font-semibold text-gray-900 text-sm">{{ role }}</span>
      <span class="text-gray-300">•</span>
      <span class="font-mono text-xs text-gray-700 font-medium">
        {{ protocol === 'RTU' ? `${protocol} ${serialPort || 'COM1'} (${baudRate || 9600})` : `${protocol} ${ip}:${port}` }}
      </span>

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
        {{ role === 'Slave' ? $t('common.listen') : $t('common.connect') }}
      </button>

      <!-- Settings Button -->
      <button
        @click="!connected && !loading && emit('open-settings')"
        :disabled="connected || loading"
        :title="connected ? $t('common.disconnectToEdit') : $t('common.settings')"
        :class="[
          'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
          connected
            ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
            : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50 cursor-pointer'
        ]"
      >
        <Settings class="w-3.5 h-3.5 text-gray-500" />
        {{ $t('common.settings') }}
      </button>
      <TcpClients v-if="role === 'Slave' && protocol === 'TCP' && connected" :key="sessionId" :session-id="sessionId" />
    </div>

    <!-- Right: Mode-Specific Controls -->
    <div class="flex items-center gap-2.5">
      <!-- SLAVE MODE CONTROLS -->
      <template v-if="role === 'Slave'">
        <!-- Auto Increment Toggle (Fixed Label, Active Emerald State) -->
        <button
          @click="emit('toggle-auto-increment')"
          :disabled="!connected || loading"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed',
            autoIncrement
              ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold'
              : 'bg-gray-100 text-gray-700 border-gray-200 hover:bg-gray-200'
          ]"
          :title="$t('common.incrementHint')"
        >
          <Repeat class="w-3.5 h-3.5 text-gray-500" />
          {{ $t('common.autoIncrement') }}
        </button>

        <!-- Random Action Button -->
        <button
          @click="emit('fill-random')"
          :disabled="!connected || loading"
          class="flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium bg-gray-100 text-gray-700 hover:bg-gray-200 transition cursor-pointer border border-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
          :title="$t('common.randomHint')"
        >
          <Shuffle class="w-3.5 h-3.5 text-gray-500" />
          {{ $t('common.random') }}
        </button>
      </template>

      <!-- MASTER MODE CONTROLS -->
      <template v-else>
        <!-- Poll Toggle (Fixed Label, Active Emerald State) -->
        <button
          @click="emit('toggle-poll')"
          :disabled="!connected || loading"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : isPolling
                ? 'bg-emerald-50 text-emerald-600 border-emerald-300 shadow-2xs font-semibold cursor-pointer'
                : 'bg-gray-100 text-gray-700 border-gray-200 hover:bg-gray-200 cursor-pointer'
          ]"
          :title="$t('common.pollHint')"
        >
          <RefreshCw :class="['w-3.5 h-3.5 text-gray-500', isPolling ? 'animate-spin' : '']" />
          {{ $t('common.poll') }}
        </button>

        <!-- Read Action Button -->
        <button
          @click="connected && emit('read-once')"
          :disabled="!connected || loading"
          :class="[
            'flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-medium border transition',
            !connected
              ? 'bg-gray-100 text-gray-400 border-gray-200 cursor-not-allowed opacity-60'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200 border-gray-200 cursor-pointer'
          ]"
          :title="$t('common.readHint')"
        >
          {{ $t('common.read') }}
        </button>
      </template>
    </div>
  </div>
</template>
