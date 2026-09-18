<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { X, CheckCircle2, AlertCircle, Loader2 } from '@lucide/vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { UpdateService } from '../services/updateService';

const props = withDefaults(
  defineProps<{
    show: boolean;
    currentVersion?: string;
    initialUpdate?: { available: boolean; version?: string; body?: string } | null;
  }>(),
  {
    currentVersion: '0.1.0',
    initialUpdate: null,
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'update-detected', info: { available: boolean; version?: string; body?: string }): void;
}>();

const isPortable = ref(false);

const checking = ref(false);
const checked = ref(false);
const updateInfo = ref<{ available: boolean; version?: string; body?: string } | null>(null);
const updateError = ref<string | null>(null);
const downloading = ref(false);
const downloadProgress = ref({ downloaded: 0, total: 0 });
const restarting = ref(false);

const progressPercent = computed(() => {
  if (downloadProgress.value.total <= 0) return 0;
  return Math.min(100, Math.round((downloadProgress.value.downloaded / downloadProgress.value.total) * 100));
});

onMounted(async () => {
  try {
    isPortable.value = await invoke<boolean>('is_portable_installation');
  } catch (e) {
    console.warn('Failed to detect installation mode:', e);
  }
});

watch(
  () => props.initialUpdate,
  (newVal) => {
    if (newVal?.available) {
      updateInfo.value = newVal;
      checked.value = true;
    }
  },
  { immediate: true }
);

async function handleCheckUpdate() {
  if (checking.value || downloading.value) return;
  checking.value = true;
  updateError.value = null;
  try {
    const res = await UpdateService.checkForUpdates();
    checked.value = true;
    if (res.available) {
      updateInfo.value = res;
      emit('update-detected', res);
    } else {
      updateInfo.value = null;
    }
  } catch (err: any) {
    updateError.value = err?.message || 'Check failed';
  } finally {
    checking.value = false;
  }
}

function getPortableDownloadUrl(): string {
  const version = updateInfo.value?.version || props.currentVersion || '0.1.2';
  const tag = version.startsWith('v') ? version : `v${version}`;
  return `https://update.dduu.cloud/dduutt/modlab/download/${tag}/Modlab-windows-x64.exe`;
}

async function handleDownloadPortable() {
  try {
    const url = getPortableDownloadUrl();
    await openUrl(url);
  } catch (err: any) {
    console.error('Failed to open download URL:', err);
    updateError.value = err?.message || 'Failed to open browser';
  }
}

async function handleApplyUpdate() {
  if (isPortable.value) {
    await handleDownloadPortable();
    emit('close');
    return;
  }

  if (downloading.value || restarting.value) return;
  downloading.value = true;
  updateError.value = null;
  try {
    await UpdateService.installUpdate((downloaded, total) => {
      downloadProgress.value = { downloaded, total };
    });
    restarting.value = true;
    await UpdateService.restartApp();
  } catch (err: any) {
    downloading.value = false;
    updateError.value = err?.message || 'Update installation failed';
  }
}
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 select-none">
    <div role="dialog" aria-modal="true" class="bg-white rounded-2xl shadow-2xl border border-gray-100 w-full max-w-sm overflow-hidden animate-in fade-in zoom-in-95 duration-150">
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 bg-gray-50/50">
        <h3 class="font-semibold text-gray-900 text-base">{{ $t('updater.title') }}</h3>
        <button
          @click="emit('close')"
          class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200/60 rounded-lg transition cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4 text-sm">
        <!-- Version & Status Card -->
        <div class="bg-gray-50 p-3.5 rounded-xl border border-gray-100 space-y-2">
          <div class="flex items-center justify-between">
            <div>
              <span class="block text-xs font-semibold text-gray-500 mb-0.5">{{ $t('updater.currentVersion') }}</span>
              <span class="font-mono text-sm font-semibold text-gray-800">
                v{{ currentVersion }}
                <span v-if="isPortable" class="font-sans font-normal text-xs text-gray-500 ml-1">({{ $t('updater.portable') }})</span>
              </span>
            </div>

            <!-- Status Badge -->
            <div v-if="checking" class="flex items-center gap-1.5 text-xs text-blue-600 font-medium">
              <Loader2 class="w-3.5 h-3.5 animate-spin" />
              <span>{{ $t('updater.checking') }}</span>
            </div>

            <div v-else-if="updateInfo?.available" class="flex items-center gap-1.5 text-xs text-emerald-700 font-medium">
              <span class="inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              <span>{{ $t('updater.available') }} v{{ updateInfo.version }}</span>
            </div>

            <div v-else-if="checked" class="flex items-center gap-1.5 text-xs text-emerald-600 font-medium">
              <CheckCircle2 class="w-3.5 h-3.5" />
              <span>{{ $t('updater.upToDate') }}</span>
            </div>
          </div>

          <!-- Error Alert -->
          <div v-if="updateError" class="flex items-center gap-1.5 text-xs text-red-600 pt-1">
            <AlertCircle class="w-3.5 h-3.5 shrink-0" />
            <span class="truncate" :title="updateError">{{ $t('updater.failed') }}</span>
          </div>
        </div>

        <!-- Release Notes (when new version is available) -->
        <div v-if="updateInfo?.available && updateInfo.body" class="space-y-1.5">
          <span class="block text-xs font-semibold text-gray-500">{{ $t('updater.releaseNotes') || 'Release Notes' }}</span>
          <div class="bg-gray-50 p-3 rounded-xl border border-gray-100 text-xs font-mono text-gray-700 whitespace-pre-wrap max-h-32 overflow-y-auto leading-relaxed">
            {{ updateInfo.body }}
          </div>
        </div>

        <!-- Download Progress (when downloading) -->
        <div v-if="downloading" class="space-y-1.5">
          <div class="flex items-center justify-between text-xs text-gray-600">
            <span>{{ $t('updater.downloading') }}</span>
            <span class="font-mono font-medium text-blue-600">{{ progressPercent }}%</span>
          </div>
          <div class="w-full bg-gray-200 h-2 rounded-full overflow-hidden">
            <div class="bg-blue-600 h-full transition-all duration-200" :style="{ width: `${progressPercent}%` }"></div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end gap-2 px-5 py-3.5 bg-gray-50/80 border-t border-gray-100">
        <button
          @click="emit('close')"
          class="px-4 py-1.5 rounded-xl text-gray-600 hover:bg-gray-200/60 font-medium transition text-sm cursor-pointer"
        >
          {{ $t('common.cancel') }}
        </button>

        <button
          v-if="updateInfo?.available"
          type="button"
          @click="handleApplyUpdate"
          :disabled="downloading || restarting"
          class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
        >
          <Loader2 v-if="downloading" class="w-3.5 h-3.5 animate-spin" />
          <span>{{ restarting ? $t('updater.restarting') : downloading ? `${$t('updater.downloading')} ${progressPercent}%` : $t('updater.updateNow') }}</span>
        </button>

        <button
          v-else
          type="button"
          @click="handleCheckUpdate"
          :disabled="checking"
          class="px-5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-md transition text-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
        >
          <Loader2 v-if="checking" class="w-3.5 h-3.5 animate-spin" />
          <span>{{ checking ? $t('updater.checking') : $t('updater.checkBtn') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
