<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getVersion } from '@tauri-apps/api/app';

import TabBar from './components/TabBar.vue';
import Toolbar from './components/Toolbar.vue';
import ConfigRow from './components/ConfigRow.vue';
import DataGrid from './components/DataGrid.vue';
import StatusBar from './components/StatusBar.vue';
import SettingsModal, { type ConnectionConfig } from './components/SettingsModal.vue';
import TrafficLog from './components/TrafficLog.vue';
import ConfirmModal from './components/ConfirmModal.vue';
import AboutModal from './components/AboutModal.vue';
import { UpdateService } from './services/updateService';
import { useSessionStore, createDefaultConnection } from './stores/sessionStore';

const store = useSessionStore();
const { locale } = useI18n();

const language = computed<'en' | 'zh'>(() => locale.value as 'en' | 'zh');
function toggleLanguage() {
  locale.value = locale.value === 'en' ? 'zh' : 'en';
}

const showSettings = ref(false);
const showAboutModal = ref(false);
const hasUpdate = ref(false);
const updateInfo = ref<{ available: boolean; version?: string; body?: string } | null>(null);
const appVersion = ref('0.1.2');
const showNewDeviceModal = ref(false);
const showConfirmModal = ref(false);
const showLogPanel = ref(false);
const tabToCloseId = ref<string | null>(null);

const activeTab = computed(() => store.activeTab);

const newDeviceDefaultConfig = computed<ConnectionConfig>(() => createDefaultConnection('Slave'));

const isGridWritable = computed(() => {
  const tab = activeTab.value;
  if (!tab || tab.busy) return false;
  if (['0x02', '0x04'].includes(tab.config.functionCode)) return false;
  if (tab.autoIncrement) return false;
  if (tab.connection.role === 'Master') {
    return Boolean(tab.connected && !tab.isPolling);
  }
  return true;
});

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch {
    // Keep fallback
  }

  setTimeout(async () => {
    try {
      const res = await UpdateService.checkForUpdates();
      if (res.available) {
        hasUpdate.value = true;
        updateInfo.value = res;
      }
    } catch {
      // Ignore background check errors
    }
  }, 3000);
});

function handleSelectTab(id: string) {
  showSettings.value = false;
  store.activeTabId = id;
}

function handleCreateDevice(newConnection: ConnectionConfig, name?: string) {
  store.addTab(newConnection, name);
  showNewDeviceModal.value = false;
}

function handleCloseTabRequest(id: string) {
  if (store.tabs.length <= 1) return;
  const tab = store.tabs.find(t => t.id === id);
  if (!tab || tab.busy) return;
  if (tab.connected) {
    tabToCloseId.value = id;
    showConfirmModal.value = true;
  } else {
    store.closeTab(id);
  }
}

async function handleConfirmCloseTab() {
  const id = tabToCloseId.value;
  showConfirmModal.value = false;
  tabToCloseId.value = null;
  if (id) {
    await store.closeTab(id);
  }
}

function handleSaveSettings(newConnection: ConnectionConfig, newName?: string) {
  if (!activeTab.value || activeTab.value.connected || activeTab.value.busy) return;
  store.updateConnection(activeTab.value.id, newConnection, newName);
  showSettings.value = false;
}
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-gray-100 font-sans overflow-hidden">
    <TabBar
      :tabs="store.tabs"
      :activeTabId="store.activeTabId"
      @select-tab="handleSelectTab"
      @add-tab="showNewDeviceModal = true"
      @close-tab="handleCloseTabRequest"
    />

    <Toolbar
      v-if="activeTab"
      :session-id="activeTab.id"
      :role="activeTab.connection.role"
      :protocol="activeTab.connection.protocol"
      :ip="activeTab.connection.ip"
      :port="activeTab.connection.port"
      :serialPort="activeTab.connection.serialPort"
      :baudRate="activeTab.connection.baudRate"
      :connected="activeTab.connected"
      :loading="activeTab.busy"
      :functionCode="activeTab.config.functionCode"
      :autoIncrement="activeTab.autoIncrement"
      :isPolling="activeTab.isPolling"
      @toggle-connect="store.toggleConnect(activeTab.id)"
      @open-settings="showSettings = true"
      @toggle-auto-increment="store.toggleAutoIncrement(activeTab.id)"
      @fill-random="store.fillRandom(activeTab.id)"
      @toggle-poll="store.togglePolling(activeTab.id)"
      @read-once="store.readOnce(activeTab.id)"
    />

    <ConfigRow
      v-if="activeTab"
      :config="activeTab.config"
      :connected="activeTab.connected"
      :busy="activeTab.busy"
      :role="activeTab.connection.role"
      @update:config="cfg => store.updateConfig(activeTab.id, cfg)"
    />

    <DataGrid
      v-if="activeTab"
      :key="activeTab.id"
      :writable="isGridWritable"
      :startAddress="activeTab.config.startAddress"
      :count="activeTab.config.count"
      :values="activeTab.values"
      :format="activeTab.config.format"
      :dataType="activeTab.config.dataType"
      :byteOrder="activeTab.config.byteOrder"
      :functionCode="activeTab.config.functionCode"
      :raw="activeTab.config.raw"
      @update-cell="(addr, val) => store.writeCell(activeTab.id, addr, val)"
      @update-cell-pair="(a1, v1, a2, v2) => store.writeCellPair(activeTab.id, a1, v1, a2, v2)"
    />

    <Transition
      enter-active-class="transition-all duration-200 ease-out overflow-hidden"
      enter-from-class="max-h-0 opacity-0"
      enter-to-class="max-h-64 opacity-100"
      leave-active-class="transition-all duration-150 ease-in overflow-hidden"
      leave-from-class="max-h-64 opacity-100"
      leave-to-class="max-h-0 opacity-0"
    >
      <TrafficLog
        v-if="activeTab && showLogPanel"
        :logs="store.allLogs"
        :devices="store.tabs.map(tab => ({ id: tab.id, title: tab.title }))"
        @clear="store.clearLogs"
        @close="showLogPanel = false"
      />
    </Transition>

    <StatusBar
      v-if="activeTab"
      :message="activeTab.statusMessage"
      :type="activeTab.statusType"
      :logCount="store.allLogs.length"
      :showLogPanel="showLogPanel"
      :language="language"
      :app-version="appVersion"
      :has-update="hasUpdate"
      @toggle-logs="showLogPanel = !showLogPanel"
      @toggle-language="toggleLanguage"
      @open-about="showAboutModal = true"
    />

    <SettingsModal
      v-if="activeTab"
      :show="showSettings"
      :config="activeTab.connection"
      :device-name="activeTab.title"
      :existing-names="store.existingNames"
      mode="edit"
      @close="showSettings = false"
      @save="handleSaveSettings"
    />

    <SettingsModal
      :show="showNewDeviceModal"
      :config="newDeviceDefaultConfig"
      :existing-names="store.existingNames"
      mode="create"
      @close="showNewDeviceModal = false"
      @save="handleCreateDevice"
    />

    <AboutModal
      :show="showAboutModal"
      :current-version="appVersion"
      :initial-update="updateInfo"
      @close="showAboutModal = false"
      @update-detected="(info) => { hasUpdate = info.available; updateInfo = info; }"
    />

    <ConfirmModal
      :show="showConfirmModal"
      :title="$t('modal.closeConnection')"
      :message="$t('modal.closeConnectionMsg')"
      @close="showConfirmModal = false"
      @confirm="handleConfirmCloseTab"
    />
  </div>
</template>
