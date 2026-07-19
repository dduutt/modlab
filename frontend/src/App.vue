<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Plus, X, Server, MonitorSmartphone, Settings, Settings2, Check } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { ConnectMaster, DisconnectMaster, ReadRegisters, WriteRegister, WriteMultipleRegisters, WriteCoil, WriteMultipleCoils, StartSlave, StopSlave, GetSlaveData, UpdateSlaveData, ClearAllConnections, GetAvailablePorts } from '../wailsjs/go/main/App'
import { EventsOn, EventsOff } from '../wailsjs/runtime/runtime'
import { formatRegisterValue, parseUserInput } from './lib/modbusFormatter'
import { reconnectAfterPollingFailure } from './lib/pollingReconnect'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

// Types
import { useModbusStore, type ModbusInstance, createDefaultInstance } from './store/modbusStore'
import { useSettingsStore, type AppLocale } from './store/settingsStore'
import { storeToRefs } from 'pinia'

const modbusStore = useModbusStore()
const settingsStore = useSettingsStore()
const { instances, activeTab } = storeToRefs(modbusStore)
const { t, locale: i18nLocale } = useI18n()

const setAppLocale = (value: unknown) => {
  if (value === 'zh-CN' || value === 'en-US') {
    settingsStore.setLocale(value as AppLocale)
  }
}

watch(() => settingsStore.locale, (nextLocale) => {
  i18nLocale.value = nextLocale
  document.documentElement.lang = nextLocale
}, { immediate: true })

const activeInstance = computed(() => instances.value.find(i => i.id === activeTab.value))

// Dynamic connection info text
const getConnectionInfoText = (inst: ModbusInstance) => {
  if (inst.protocol === 'tcp') return `TCP ${inst.tcpConfig.ip}:${inst.tcpConfig.port}`
  return `RTU ${inst.rtuConfig.port} @ ${inst.rtuConfig.baudRate}`
}

// Dialog States
const showAddDialog = ref(false)
const showConnectionDialog = ref(false)
const showWriteDialog = ref(false)
const showSettingsDialog = ref(false)

// Connection Setup temporary state
const tempConnectionConfig = ref<any>({})

// Available Serial Ports
const availablePorts = ref<string[]>([])
let portPollingTimer: any = null

const syncSelectedSerialPort = () => {
  if (!tempConnectionConfig.value?.rtuConfig) return

  if (availablePorts.value.length === 0) {
    tempConnectionConfig.value.rtuConfig.port = ''
    return
  }

  if (!availablePorts.value.includes(tempConnectionConfig.value.rtuConfig.port)) {
    tempConnectionConfig.value.rtuConfig.port = availablePorts.value[0]
  }
}

const fetchPorts = async () => {
  try {
    const ports = await GetAvailablePorts()
    availablePorts.value = ports || []
    syncSelectedSerialPort()
  } catch(e) {
    console.error("Failed to fetch ports", e)
  }
}

const startPortPolling = () => {
  stopPortPolling()
  fetchPorts()
  portPollingTimer = setInterval(fetchPorts, 2000)
}

const stopPortPolling = () => {
  if (!portPollingTimer) return
  clearInterval(portPollingTimer)
  portPollingTimer = null
}

const canSaveConnection = computed(() => {
  if (tempConnectionConfig.value?.protocol !== 'rtu') return true
  return availablePorts.value.includes(tempConnectionConfig.value?.rtuConfig?.port)
})

// System Status
type StatusType = 'info' | 'success' | 'error'
type StatusParams = Record<string, string | number>
type StatusState = { key: string, params?: StatusParams, type: StatusType }

const systemStatus = ref<StatusState>({ key: 'status.ready', type: 'info' })
const setStatus = (key: string, type: StatusType = 'info', params?: StatusParams) => {
  systemStatus.value = { key, params, type }
}

const getErrorMessage = (error: unknown) => error instanceof Error ? error.message : String(error)

const markMasterError = async (inst: ModbusInstance) => {
  if (inst.type !== 'master') return
  inst.status = 'disconnected'
  inst.hasError = true
  inst.isAutoRead = false
  stopPolling(inst.id)
  try {
    await DisconnectMaster(inst.id)
  } catch (disconnectErr) {
    console.error(disconnectErr)
  }
}

const openConnectionDialog = async () => {
  if (!activeInstance.value) return
  // Clone current config to temp state
  tempConnectionConfig.value = JSON.parse(JSON.stringify({
    protocol: activeInstance.value.protocol,
    tcpConfig: activeInstance.value.tcpConfig,
    rtuConfig: activeInstance.value.rtuConfig
  }))
  showConnectionDialog.value = true
}

const saveConnectionConfig = () => {
  if (!activeInstance.value) return
  if (!canSaveConnection.value) {
    setStatus('messages.noSerialPorts', 'error')
    return
  }
  activeInstance.value.protocol = tempConnectionConfig.value.protocol
  activeInstance.value.tcpConfig = tempConnectionConfig.value.tcpConfig
  activeInstance.value.rtuConfig = tempConnectionConfig.value.rtuConfig
  showConnectionDialog.value = false
}

const activeTimers = new Map<string, any>()

const startPolling = (inst: ModbusInstance) => {
  stopPolling(inst.id)
  const timer = setInterval(async () => {
    try {
      const res = await ReadRegisters(inst.id, inst.slaveId, inst.functionCode, inst.startAddress, inst.count)
      if (res && res.length) {
        for(let i=0; i<res.length; i++) {
          inst.data[i] = res[i]
        }
        setStatus('status.autoReadSuccess', 'success', { name: inst.name, count: inst.count })
        inst.status = 'connected' // Recovered!
        inst.hasError = false
      }
    } catch (e) {
      setStatus('status.autoReadError', 'error', { name: inst.name, error: getErrorMessage(e) })
      inst.status = 'disconnected' // Reflect broken state in UI
      inst.hasError = true
      
      // Reconnect only while this exact polling session is still active.
      try {
        await reconnectAfterPollingFailure({
          isActive: () => inst.isAutoRead && activeTimers.get(inst.id) === timer,
          disconnect: () => DisconnectMaster(inst.id),
          reconnect: () => {
            if (inst.protocol === 'tcp') {
              return ConnectMaster(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
            }
            return ConnectMaster(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
          },
        })
      } catch (reconnectErr) {
        if (inst.isAutoRead) {
          setStatus('status.disconnectFailed', 'error', { name: inst.name, error: getErrorMessage(reconnectErr) })
        }
      }
    }
  }, inst.intervalMs || 1000)
  activeTimers.set(inst.id, timer)
}

const stopPolling = (id: string) => {
  if (activeTimers.has(id)) {
    clearInterval(activeTimers.get(id))
    activeTimers.delete(id)
  }
}

const stopAutoRead = (inst: ModbusInstance) => {
  inst.isAutoRead = false
  stopPolling(inst.id)
}

const toggleAutoRead = (inst: ModbusInstance) => {
  inst.isAutoRead = !inst.isAutoRead
  if (inst.isAutoRead) {
    startPolling(inst)
    setStatus('status.autoReadStarted', 'info', { name: inst.name })
  } else {
    stopPolling(inst.id)
    setStatus('status.autoReadStopped', 'info', { name: inst.name })
  }
}

const readOnce = async (inst: ModbusInstance) => {
    try {
      if (inst.type === 'master') {
        const res = await ReadRegisters(inst.id, inst.slaveId, inst.functionCode, inst.startAddress, inst.count)
        if (res && res.length) {
          for(let i=0; i<res.length; i++) {
            inst.data[i] = res[i]
          }
          setStatus('status.readSuccess', 'success', { name: inst.name, count: inst.count })
        }
      } else {
        const res = await GetSlaveData(inst.id, inst.startAddress, inst.count, inst.functionCode)
        if (res && res.length) {
          for(let i=0; i<res.length; i++) {
            inst.data[i] = res[i]
          }
          setStatus('status.memoryRefreshed', 'success', { name: inst.name })
        }
      }
    } catch (e) {
      await markMasterError(inst)
      setStatus('status.readError', 'error', { name: inst.name, error: getErrorMessage(e) })
    }
}

const toggleConnection = async () => {
  if (!activeInstance.value) return
  const inst = activeInstance.value
  if (inst.status === 'connected') {
    stopAutoRead(inst)
    inst.isAutoIncrement = false
    try {
      if (inst.type === 'master') {
        await DisconnectMaster(inst.id)
      } else {
        await StopSlave(inst.id)
      }
      inst.status = 'disconnected'
      inst.hasError = false
      setStatus('status.disconnected', 'info', { name: inst.name })
    } catch (e) {
      inst.status = 'disconnected'
      inst.hasError = true
      setStatus('status.disconnectFailed', 'error', { name: inst.name, error: getErrorMessage(e) })
    }
  } else {
    try {
      if (inst.type === 'master') {
        if (inst.protocol === 'tcp') {
          await ConnectMaster(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
          setStatus('status.connectedTcp', 'success', { name: inst.name, address: `${inst.tcpConfig.ip}:${inst.tcpConfig.port}` })
        } else {
          await ConnectMaster(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
          setStatus('status.connectedRtu', 'success', { name: inst.name, address: inst.rtuConfig.port })
        }
      } else {
        // SLAVE LOGIC
        if (inst.protocol === 'tcp') {
          await StartSlave(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
          setStatus('status.listeningTcp', 'success', { name: inst.name, address: `${inst.tcpConfig.ip}:${inst.tcpConfig.port}` })
        } else {
          await StartSlave(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
          setStatus('status.listeningRtu', 'success', { name: inst.name, address: inst.rtuConfig.port })
        }
      }
      inst.status = 'connected'
      inst.hasError = false
    } catch (e) {
      inst.hasError = true
      setStatus('status.connectionFailed', 'error', { name: inst.name, error: getErrorMessage(e) })
    }
  }
}

// Write Dialog temporary state
const writeTarget = ref({ address: 0, currentValue: '' as string | number, newValue: '' as string | number })

const openWriteDialog = (address: number, currentValue: string | number) => {
  if (!activeInstance.value) return
  if (activeInstance.value.status !== 'connected') {
    setStatus('status.connectBeforeWrite', 'error', { name: activeInstance.value.name })
    return
  }
  if (activeInstance.value.functionCode === '02' || activeInstance.value.functionCode === '04') {
    alert(t('messages.readOnly'))
    return
  }
  writeTarget.value = { address, currentValue, newValue: currentValue }
  showWriteDialog.value = true
}

const commitWrite = async () => {
  const inst = activeInstance.value
  if (!inst) return

  let parsedValues: number[]

  try {
    parsedValues = parseUserInput(
      String(writeTarget.value.newValue),
      inst.dataType,
      inst.format,
      inst.byteOrder,
      inst.functionCode
    )
  } catch (e) {
    setStatus('status.invalidInput', 'error', { name: inst.name, error: getErrorMessage(e) })
    return
  }

  try {
    if (inst.type === 'master') {
      const isCoil = inst.functionCode === '01'
      if (parsedValues.length === 1) {
        if (isCoil) {
          await WriteCoil(inst.id, inst.slaveId, writeTarget.value.address, parsedValues[0])
        } else {
          await WriteRegister(inst.id, inst.slaveId, writeTarget.value.address, parsedValues[0])
        }
      } else {
        if (isCoil) {
          await WriteMultipleCoils(inst.id, inst.slaveId, writeTarget.value.address, parsedValues)
        } else {
          await WriteMultipleRegisters(inst.id, inst.slaveId, writeTarget.value.address, parsedValues)
        }
      }
    } else {
      await UpdateSlaveData(inst.id, writeTarget.value.address, parsedValues, inst.functionCode)
      setStatus('status.updatedMemory', 'success', { name: inst.name, address: writeTarget.value.address })
    }
    readOnce(inst)
  } catch (e) {
    if (inst.type === 'master') {
      await markMasterError(inst)
    }
    setStatus('status.writeFailed', 'error', { name: inst.name, error: getErrorMessage(e) })
  }
  showWriteDialog.value = false
}

const onAddressingChange = async (inst: ModbusInstance) => {
  if (inst.type === 'slave' && inst.status === 'connected') {
    try {
      const res = await GetSlaveData(inst.id, inst.startAddress, inst.count, inst.functionCode)
      if (res && res.length) {
        for(let i=0; i<res.length; i++) {
          inst.data[i] = res[i]
        }
        // zero out the rest up to count just in case
        for(let i=res.length; i<inst.count; i++) {
          inst.data[i] = 0
        }
      }
    } catch (e) {
      console.error("Failed to fetch slave data on addressing change", e)
    }
  }
}

const randomizeSlave = async (inst: ModbusInstance) => {
  if (inst.status !== 'connected') return
  const isCoil = inst.functionCode === '01' || inst.functionCode === '02'
  const is32Bit = !isCoil && (inst.dataType === 'Int32' || inst.dataType === 'UInt32' || inst.dataType === 'Float32')
  
  const randomValues: number[] = []
  
  if (isCoil) {
    for (let i = 0; i < inst.count; i++) {
      randomValues.push(Math.random() > 0.5 ? 1 : 0)
    }
  } else {
    let i = 0
    while (i < inst.count) {
      let valStr = '0'
      if (inst.dataType === 'Int16') {
        valStr = String(Math.floor(Math.random() * 65536) - 32768)
      } else if (inst.dataType === 'UInt16') {
        valStr = String(Math.floor(Math.random() * 65536))
      } else if (inst.dataType === 'Int32') {
        valStr = String(Math.floor(Math.random() * 4294967296) - 2147483648)
      } else if (inst.dataType === 'UInt32') {
        valStr = String(Math.floor(Math.random() * 4294967296))
      } else if (inst.dataType === 'Float32') {
        valStr = String((Math.random() * 10000 - 5000).toFixed(4))
      }
      
      try {
        const parsed = parseUserInput(valStr, inst.dataType, 'Dec', inst.byteOrder, inst.functionCode)
        if (is32Bit) {
          randomValues.push(parsed[0])
          if (i + 1 < inst.count) {
            randomValues.push(parsed[1])
          }
          i += 2
        } else {
          randomValues.push(parsed[0])
          i += 1
        }
      } catch (e) {
        // Fallback safety
        randomValues.push(0)
        i += 1
      }
    }
  }

  try {
    await UpdateSlaveData(inst.id, inst.startAddress, randomValues, inst.functionCode)
    inst.data = randomValues
    setStatus('status.randomized', 'success', { name: inst.name, count: inst.count, dataType: inst.dataType })
  } catch (e: any) {
    setStatus('status.randomizeError', 'error', { name: inst.name, error: getErrorMessage(e) })
  }
}

const incrementSlave = async (inst: ModbusInstance) => {
  if (inst.status !== 'connected') return
  const isCoil = inst.functionCode === '01' || inst.functionCode === '02'
  const is32Bit = !isCoil && (inst.dataType === 'Int32' || inst.dataType === 'UInt32' || inst.dataType === 'Float32')
  
  const newValues: number[] = []
  
  if (isCoil) {
    for (let i = 0; i < inst.count; i++) {
      newValues.push(inst.data[i] === 1 ? 0 : 1)
    }
  } else {
    let i = 0
    while (i < inst.count) {
      let currentStr = formatRegisterValue(inst.data, i, inst.dataType, 'Dec', inst.byteOrder, inst.functionCode)
      if (currentStr === '---' || currentStr === '-') currentStr = '0'
      
      let num = parseFloat(currentStr)
      if (isNaN(num)) num = 0
      
      if (inst.dataType === 'Float32') {
        num += 1.5
      } else {
        num += 1
      }
      
      let newStr = String(num)
      
      try {
        const parsed = parseUserInput(newStr, inst.dataType, 'Dec', inst.byteOrder, inst.functionCode)
        if (is32Bit) {
          newValues.push(parsed[0])
          if (i + 1 < inst.count) {
            newValues.push(parsed[1])
          }
          i += 2
        } else {
          newValues.push(parsed[0])
          i += 1
        }
      } catch (e) {
        newValues.push(0)
        i += 1
      }
    }
  }

  try {
    await UpdateSlaveData(inst.id, inst.startAddress, newValues, inst.functionCode)
    inst.data = newValues
  } catch (e: any) {
    setStatus('status.incrementError', 'error', { name: inst.name, error: getErrorMessage(e) })
  }
}

const startAutoIncrement = (inst: ModbusInstance) => {
  stopPolling(inst.id)
  const timer = setInterval(() => {
    incrementSlave(inst)
  }, inst.intervalMs || 1000)
  activeTimers.set(inst.id, timer)
}

const toggleAutoIncrement = (inst: ModbusInstance) => {
  inst.isAutoIncrement = !inst.isAutoIncrement
  if (inst.isAutoIncrement) {
    startAutoIncrement(inst)
    setStatus('status.incrementStarted', 'info', { name: inst.name })
  } else {
    stopPolling(inst.id)
    setStatus('status.incrementStopped', 'info', { name: inst.name })
  }
}


// Listen for external Master writing to our Server
onMounted(() => {
  fetchPorts()
  // Always nuke existing connections on fresh load/refresh to prevent zombie ports
  ClearAllConnections().catch(console.error)

  EventsOn('slave_write', (id: string, addr: number, args: number[]) => {
    const inst = instances.value.find(i => i.id === id)
    if (inst && inst.type === 'slave') {
      // If the write falls within the currently displayed window, update the grid!
      const start = inst.startAddress
      const end = start + inst.count
      let updated = false
      for (let i = 0; i < args.length; i++) {
        const targetAddr = addr + i
        if (targetAddr >= start && targetAddr < end) {
          inst.data[targetAddr - start] = args[i]
          updated = true
        }
      }
      if (updated) {
        setStatus('status.externalWrite', 'success', { name: inst.name, address: addr })
      }
    }
  })
})

onUnmounted(() => {
  stopPortPolling()
  EventsOff('slave_write')
})

// Instance Management
const newInstanceName = ref('')
const nameError = ref('')

watch(showAddDialog, (val) => {
  if (val) {
    newInstanceName.value = ''
    nameError.value = ''
  }
})

watch(newInstanceName, () => {
  if (nameError.value) nameError.value = ''
})

watch(showConnectionDialog, (val) => {
  if (val) {
    startPortPolling()
  } else {
    stopPortPolling()
  }
})

watch(() => tempConnectionConfig.value?.protocol, (protocol) => {
  if (showConnectionDialog.value && protocol === 'rtu') {
    fetchPorts()
    syncSelectedSerialPort()
  }
})

const addInstance = (type: 'master' | 'slave') => {
  const maxId = instances.value.reduce((max, inst) => Math.max(max, parseInt(inst.id) || 0), 0)
  const newId = String(maxId + 1)
  const name = newInstanceName.value.trim() || `${type === 'master' ? 'Master' : 'Slave'} ${newId}`

  if (instances.value.some(inst => inst.name.toLowerCase() === name.toLowerCase())) {
    nameError.value = 'messages.duplicateName'
    return
  }

  const newInst = createDefaultInstance(newId, type)
  newInst.name = name
  instances.value.push(newInst)
  activeTab.value = newId
  showAddDialog.value = false
  newInstanceName.value = '' // Reset
}

const removeInstance = (id: string, event: Event) => {
  event.stopPropagation()
  if (instances.value.length === 1) return
  
  stopPolling(id)
  const instToClose = instances.value.find(i => i.id === id)
  if (instToClose) {
    instToClose.isAutoRead = false
    if (instToClose.type === 'master') {
      DisconnectMaster(id).catch((e: any) => console.error(e))
    } else {
      StopSlave(id).catch((e: any) => console.error(e))
    }
  }
  
  const index = instances.value.findIndex(i => i.id === id)
  instances.value.splice(index, 1)
  if (activeTab.value === id) {
    activeTab.value = instances.value[Math.max(0, index - 1)]?.id || instances.value[0].id
  }
}

const getMatrixRows = (instance: ModbusInstance) => {
  const rowCount = Math.ceil(instance.count / 10)
  const isCoil = instance.functionCode === '01' || instance.functionCode === '02'
  const is32Bit = !isCoil && (instance.dataType === 'Int32' || instance.dataType === 'UInt32' || instance.dataType === 'Float32')
  
  return Array.from({ length: rowCount }).map((_, rIndex) => {
    return Array.from({ length: 10 }).map((_, cIndex) => {
      const addr = rIndex * 10 + cIndex
      let val = null
      let writable = addr < instance.count
      if (addr < instance.count) {
        if (is32Bit && addr % 2 !== 0) {
          val = '-'
          writable = false
        } else {
          val = formatRegisterValue(instance.data, addr, instance.dataType, instance.format, instance.byteOrder, instance.functionCode)
        }
      }
      let rawVal = ''
      if (addr < instance.count && !isCoil && instance.data[addr] !== undefined) {
        rawVal = '0x' + (instance.data[addr] & 0xFFFF).toString(16).toUpperCase().padStart(4, '0')
      }
      return {
        address: addr,
        value: val,
        rawValue: rawVal,
        writable
      }
    })
  })
}
</script>

<template>
  <TooltipProvider>
    <div class="flex flex-col h-screen bg-muted/20 text-foreground font-sans selection:bg-primary/20">
    <!-- Main Content Layout -->
    <div class="flex-1 flex overflow-hidden">
      
      <!-- Main Work Area -->
      <main class="flex-1 flex flex-col min-w-0">
        <Tabs v-model="activeTab" class="flex-1 flex flex-col overflow-hidden relative z-10">
          
          <!-- Row 1: Top Tab View -->
          <div class="flex items-end px-4 pt-3 border-b border-border z-10 shrink-0">
            <div class="w-full overflow-x-auto no-scrollbar flex items-end">
              <TabsList class="bg-transparent h-auto p-0 flex items-end justify-start gap-1 rounded-none border-0">
                  <TabsTrigger 
                    v-for="instance in instances" :key="instance.id" :value="instance.id"
                    class="group relative flex items-center gap-2 px-4 py-2 text-[13px] transition-all duration-200 border border-b-0 rounded-t-lg outline-none cursor-pointer -mb-[1px] data-[state=active]:bg-card data-[state=active]:text-primary data-[state=active]:border-border data-[state=active]:border-t-2 data-[state=active]:border-t-primary data-[state=active]:z-10 data-[state=inactive]:border-transparent data-[state=inactive]:bg-transparent data-[state=inactive]:text-muted-foreground hover:data-[state=inactive]:bg-muted/60"
                  >
                  <!-- Three-color Status Light -->
                  <div class="h-2 w-2 rounded-full shrink-0 transition-all duration-300" 
                       :class="{
                         'bg-muted-foreground/30': instance.status === 'disconnected' && !instance.hasError,
                         'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]': instance.status === 'connected' && !instance.hasError,
                         'bg-destructive shadow-[0_0_6px_rgba(239,68,68,0.5)]': instance.hasError
                       }">
                  </div>
                  
                  <span class="font-medium tracking-wide max-w-[140px] truncate select-none transition-colors" :class="activeTab === instance.id ? 'text-foreground' : ''">{{ instance.name }}</span>
                  
                  <div 
                    v-if="instances.length > 1"
                    @click.stop="removeInstance(instance.id, $event)"
                    class="w-4 h-4 ml-1.5 shrink-0 rounded-md flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-destructive/15 hover:text-destructive transition-colors duration-200"
                     :title="t('common.close')"
                  >
                    <X class="w-3 h-3" />
                  </div>
                  <div v-else class="w-4 h-4 ml-1.5 shrink-0"></div>
                </TabsTrigger>

              <!-- Add Button -->
              <div class="flex items-center justify-center h-full pb-1 pl-1 pr-4 z-10 transition-all duration-300">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" @click="showAddDialog = true" class="h-7 w-7 text-muted-foreground hover:text-foreground hover:bg-muted/60 shrink-0 group">
                      <Plus class="w-3.5 h-3.5 transition-transform duration-500 ease-out group-hover:rotate-90 group-hover:text-primary group-hover:scale-110" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{{ t('actions.addConnection') }}</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </TabsList>
          </div>
        </div>

        <!-- Render active tab content -->
          <TabsContent 
            v-for="instance in instances" 
            :key="instance.id" 
            :value="instance.id" 
            class="flex-1 flex flex-col min-h-0 m-0 outline-none bg-card"
          >
            
            <!-- Row 2: Master 配置信息一行 (Master Configuration) -->
            <div class="px-6 py-3 border-b border-border flex items-center justify-between gap-4 shrink-0 bg-card">
            <div class="flex items-center gap-3 shrink-0">
              <h2 class="text-sm font-semibold flex items-center gap-2">
                <span class="opacity-80">{{ instance.type === 'master' ? t('connection.master') : t('connection.slave') }}</span>
                <span class="text-muted-foreground font-normal">•</span> 
                <span class="font-mono text-[13px] text-foreground">{{ getConnectionInfoText(instance) }}</span>
              </h2>

              <Button 
                variant="outline"
                size="sm" 
                class="min-w-[110px] transition-colors ml-2"
                @click="toggleConnection"
              >
                {{ instance.status === 'connected' ? t('connection.disconnect') : t('connection.connect') }}
              </Button>

              <Button variant="outline" size="sm" class="min-w-[90px]" @click="openConnectionDialog" :disabled="instance.status === 'connected'">
                <Settings2 class="w-4 h-4 mr-2 text-muted-foreground" /> {{ t('connection.settings') }}
              </Button>
            </div>

            <div class="flex items-center gap-3 shrink-0 ml-auto">
              <Button 
                v-if="instance.type === 'slave'"
                variant="outline"
                size="sm"
                class="min-w-[100px]"
                :disabled="instance.status !== 'connected'"
                @click="randomizeSlave(instance)"
              >
                {{ t('actions.random') }}
              </Button>

              <Button 
                v-if="instance.type === 'slave'"
                variant="outline"
                size="sm"
                class="min-w-[130px] transition-colors" 
                @click="toggleAutoIncrement(instance)"
                :disabled="instance.status !== 'connected'"
              >
                {{ instance.isAutoIncrement ? t('actions.stopIncrement') : t('actions.increment') }}
              </Button>


              <Button 
                v-if="instance.type === 'master'"
                variant="outline"
                size="sm"
                class="min-w-[130px] transition-colors" 
                @click="toggleAutoRead(instance)"
                :disabled="instance.status !== 'connected'"
              >
                {{ instance.isAutoRead ? t('actions.stopPoll') : t('actions.startPoll') }}
              </Button>
              
              <Button 
                v-if="instance.type === 'master'"
                variant="outline"
                size="sm"
                class="min-w-[100px]"
                :disabled="instance.status !== 'connected' || instance.isAutoRead"
                @click="readOnce(instance)"
              >
                {{ t('actions.read') }}
              </Button>
            </div>
            </div>

            <!-- Row 3: 采集配置一行 (Collection Configuration) -->
            <div class="px-6 py-3 border-b border-border flex flex-wrap items-center justify-between gap-4 shrink-0">
              
              <!-- Left Side: Parameter Groups -->
              <div class="flex flex-wrap items-center gap-y-4 gap-x-5">
              
              <!-- Group 1: Addressing -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[70px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">{{ t('fields.unitId') }}</Label>
                  <Input v-model="instance.slaveId" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" />
                </div>
                <div class="space-y-1.5 w-[140px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">{{ t('fields.function') }}</Label>
                  <Select :key="settingsStore.locale" v-model="instance.functionCode" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @update:modelValue="onAddressingChange(instance)">
                    <SelectTrigger class="h-8 text-xs bg-background shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs">
                      <SelectItem value="01">{{ t('functionCodes.01') }}</SelectItem>
                      <SelectItem value="02">{{ t('functionCodes.02') }}</SelectItem>
                      <SelectItem value="03">{{ t('functionCodes.03') }}</SelectItem>
                      <SelectItem value="04">{{ t('functionCodes.04') }}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-1.5 w-[80px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">{{ t('fields.start') }}</Label>
                  <Input v-model="instance.startAddress" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @change="onAddressingChange(instance)" />
                </div>
                <div class="space-y-1.5 w-[70px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">{{ t('fields.count') }}</Label>
                  <Input v-model="instance.count" type="number" max="100" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @change="onAddressingChange(instance)" />
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 2: Parsing -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[100px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">{{ t('fields.dataType') }}</Label>
                  <Select v-model="instance.dataType" :disabled="instance.functionCode === '01' || instance.functionCode === '02'">
                    <SelectTrigger class="h-8 text-xs bg-background shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs">
                      <SelectItem value="Int16">Int16</SelectItem>
                      <SelectItem value="UInt16">UInt16</SelectItem>
                      <SelectItem value="Int32">Int32</SelectItem>
                      <SelectItem value="UInt32">UInt32</SelectItem>
                      <SelectItem value="Float32">Float32</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-1.5 w-[80px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">{{ t('fields.format') }}</Label>
                  <Select v-model="instance.format" :disabled="instance.functionCode === '01' || instance.functionCode === '02'">
                    <SelectTrigger class="h-8 text-xs bg-background shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs">
                      <SelectItem value="Dec">Dec</SelectItem>
                      <SelectItem value="Hex">Hex</SelectItem>
                      <SelectItem value="Bin">Bin</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-1.5 w-[90px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">{{ t('fields.byteOrder') }}</Label>
                  <Select v-model="instance.byteOrder" :disabled="instance.functionCode === '01' || instance.functionCode === '02'">
                    <SelectTrigger class="h-8 text-xs bg-background font-mono shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs font-mono">
                      <SelectItem value="ABCD">ABCD</SelectItem>
                      <SelectItem value="CDAB">CDAB</SelectItem>
                      <SelectItem value="DCBA">DCBA</SelectItem>
                      <SelectItem value="BADC">BADC</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 3: Polling Settings -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[90px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">{{ t('fields.interval') }}</Label>
                  <Input v-model="instance.intervalMs" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" />
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 4: Display Options -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 flex flex-col items-center justify-center">
                   <Label class="text-[11px] font-semibold text-muted-foreground cursor-pointer" @click="modbusStore.toggleRawData()">{{ t('fields.raw') }}</Label>
                  <div class="h-8 flex items-center">
                    <button 
                      @click="modbusStore.toggleRawData()" 
                      type="button" 
                      class="relative inline-flex h-4 w-7 shrink-0 cursor-pointer items-center justify-center rounded-full border-2 border-transparent focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background transition-colors"
                      :class="modbusStore.showRawData ? 'bg-primary' : 'bg-input'"
                    >
                       <span class="sr-only">{{ t('fields.raw') }}</span>
                      <span 
                        class="pointer-events-none inline-block h-3 w-3 transform rounded-full bg-background shadow-lg ring-0 transition duration-200 ease-in-out"
                        :class="modbusStore.showRawData ? 'translate-x-[0.35rem]' : '-translate-x-[0.35rem]'"
                      />
                    </button>
                  </div>
                </div>
              </div>
              </div>
            </div>

            <!-- Row 4: Data Matrix (Matrix Display) -->
            <div class="flex-1 p-6 relative flex flex-col min-h-0">
              <div class="rounded-md border border-border flex-1 overflow-hidden flex flex-col bg-background">
                <!-- Table Content with sticky header -->
                <div class="flex-1 overflow-auto relative">
                  <Table class="w-full text-sm">
                    <TableHeader class="bg-muted/30 sticky top-0 z-10">
                      <TableRow class="hover:bg-transparent border-none">
                        <TableHead class="w-24 text-center border-b border-r border-border font-bold text-foreground">{{ t('common.address') }}</TableHead>
                        <TableHead v-for="i in 10" :key="i" class="text-center font-bold text-foreground w-[9%] border-b border-r border-border">
                          {{ i - 1 }}
                        </TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      <TableRow 
                        v-for="(row, rIdx) in getMatrixRows(instance)" 
                        :key="rIdx"
                        class="transition-colors duration-150 border-none"
                        :class="{ 'bg-muted/10': rIdx % 2 !== 0 }"
                      >
                        <!-- Base Address Column -->
                        <TableCell class="font-mono text-primary font-semibold text-center border-b border-r border-border bg-muted/10">
                          {{ instance.startAddress + rIdx * 10 }}
                        </TableCell>
                        
                        <!-- Data Cells -->
                        <TableCell v-for="(cell, cIdx) in row" :key="cIdx" class="p-0 border-b border-r border-border">
                          <Tooltip v-if="cell.value !== null && cell.writable">
                            <TooltipTrigger asChild>
                              <button 
                                @click="openWriteDialog(instance.startAddress + rIdx * 10 + cIdx, cell.value)"
                                class="w-full h-auto min-h-[36px] py-1 font-mono text-center rounded-none bg-transparent hover:bg-muted/50 focus:bg-primary/10 focus:text-primary focus:ring-1 focus:ring-primary/50 focus:z-10 relative transition-colors text-foreground flex flex-col items-center justify-center gap-0.5"
                              >
                                <span class="leading-none">{{ cell.value }}</span>
                                <span v-if="modbusStore.showRawData && cell.rawValue" class="text-[10px] text-muted-foreground/70 leading-none">{{ cell.rawValue }}</span>
                              </button>
                            </TooltipTrigger>
                            <TooltipContent>
                              <p>{{ t('data.writeAddress') }} <span class="font-mono text-primary">{{ instance.startAddress + rIdx * 10 + cIdx }}</span></p>
                            </TooltipContent>
                          </Tooltip>
                          <div v-else-if="cell.value !== null" class="w-full min-h-[36px] py-1 font-mono text-center text-muted-foreground flex flex-col items-center justify-center gap-0.5">
                            <span class="leading-none">{{ cell.value }}</span>
                            <span v-if="modbusStore.showRawData && cell.rawValue" class="text-[10px] text-muted-foreground/70 leading-none">{{ cell.rawValue }}</span>
                          </div>
                          <div v-else class="w-full h-8 bg-muted/5"></div>
                        </TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
                </div>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </main>
    </div>

      <!-- Bottom Status Bar -->
      <footer class="h-10 bg-card border-t border-border flex items-center justify-between px-4 shrink-0 text-xs">
        <div class="flex items-center gap-4 text-xs">
          <span 
            class="font-medium flex items-center gap-2"
            :class="{
              'text-muted-foreground': systemStatus.type === 'info',
              'text-emerald-500': systemStatus.type === 'success',
              'text-destructive': systemStatus.type === 'error'
            }"
          >
            <div 
              class="h-1.5 w-1.5 rounded-full"
              :class="{
                'bg-primary/50 animate-pulse': systemStatus.type === 'info',
                'bg-emerald-500': systemStatus.type === 'success',
                'bg-destructive': systemStatus.type === 'error'
              }"
            ></div>
            {{ t(systemStatus.key, systemStatus.params || {}) }}
          </span>
        </div>
        <div class="flex items-center gap-3">
          <Button variant="ghost" size="sm" class="h-7 px-2 text-xs text-muted-foreground hover:text-foreground" @click="showSettingsDialog = true">
            <Settings class="w-3.5 h-3.5 mr-1.5" />
            {{ t('common.settings') }}
          </Button>
          <span class="text-muted-foreground whitespace-nowrap">{{ t('app.copyright') }}</span>
        </div>
      </footer>

      <!-- Application Settings Dialog -->
      <Dialog v-model:open="showSettingsDialog">
        <DialogContent class="sm:max-w-[360px]">
          <DialogHeader>
            <DialogTitle>{{ t('dialogs.settings') }}</DialogTitle>
            <DialogDescription>{{ t('dialogs.settingsDescription') }}</DialogDescription>
          </DialogHeader>
          <div class="grid gap-2 py-4">
            <Label>{{ t('dialogs.language') }}</Label>
            <Select :model-value="settingsStore.locale" @update:modelValue="setAppLocale">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="zh-CN">{{ t('dialogs.chinese') }}</SelectItem>
                <SelectItem value="en-US">{{ t('dialogs.english') }}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <DialogFooter>
            <Button variant="outline" @click="showSettingsDialog = false">{{ t('common.close') }}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Add Instance Dialog -->
      <Dialog v-model:open="showAddDialog">
        <DialogContent class="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{{ t('dialogs.addConnection') }}</DialogTitle>
            <DialogDescription>{{ t('dialogs.chooseRole') }}</DialogDescription>
          </DialogHeader>
          <div class="grid gap-4 py-4">
            <div class="space-y-2 mb-2">
              <Label :class="{ 'text-destructive': nameError }">{{ t('dialogs.name') }}</Label>
              <Input 
                v-model="newInstanceName" 
                :placeholder="t('placeholders.optionalName')"
                autofocus 
                @keyup.enter="addInstance('master')" 
                :class="{ 'border-destructive focus-visible:ring-destructive': nameError }"
              />
              <p v-if="nameError" class="text-[11px] font-medium text-destructive mt-1">{{ t(nameError) }}</p>
            </div>
            
            <Label class="text-muted-foreground">{{ t('dialogs.role') }}</Label>
            <div class="grid grid-cols-2 gap-4">
              <Button 
                variant="outline" 
                class="h-24 flex flex-col items-center justify-center gap-2 hover:border-primary hover:text-primary transition-colors"
                @click="addInstance('master')"
              >
                <MonitorSmartphone class="w-8 h-8" />
                <span class="font-semibold">{{ t('connection.master') }}</span>
              </Button>
              <Button 
                variant="outline" 
                class="h-24 flex flex-col items-center justify-center gap-2 hover:border-primary hover:text-primary transition-colors"
                @click="addInstance('slave')"
              >
                <Server class="w-8 h-8" />
                <span class="font-semibold">{{ t('connection.slave') }}</span>
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <!-- Connection Setup Dialog -->
      <Dialog v-model:open="showConnectionDialog">
        <DialogContent class="sm:w-[480px] sm:max-w-[480px] sm:h-[440px] grid-rows-[auto_1fr_auto]">
          <DialogHeader>
            <DialogTitle>{{ t('dialogs.connection') }}</DialogTitle>
            <DialogDescription>{{ t('dialogs.connectionDescription') }}</DialogDescription>
          </DialogHeader>
          
          <Tabs v-model="tempConnectionConfig.protocol" class="mt-2 min-h-0">
            <TabsList class="grid w-full grid-cols-2">
              <TabsTrigger value="tcp">Modbus TCP</TabsTrigger>
              <TabsTrigger value="rtu">Modbus RTU</TabsTrigger>
            </TabsList>
            
            <!-- TCP Config -->
            <TabsContent value="tcp" class="space-y-4 pt-4">
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">{{ t('connection.ip') }}</Label>
                <Input v-model="tempConnectionConfig.tcpConfig.ip" class="col-span-3 font-mono" />
              </div>
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">{{ t('common.port') }}</Label>
                <Input v-model="tempConnectionConfig.tcpConfig.port" type="number" class="col-span-3 font-mono" />
              </div>
            </TabsContent>
            
            <!-- RTU Config -->
            <TabsContent value="rtu" class="space-y-4 pt-4">
              <div class="space-y-2">
                <Label>{{ t('common.port') }}</Label>
                <Select v-model="tempConnectionConfig.rtuConfig.port" :disabled="availablePorts.length === 0">
                  <SelectTrigger class="font-mono">
                    <SelectValue :placeholder="availablePorts.length === 0 ? t('placeholders.noSerialPorts') : t('placeholders.selectPort')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="port in availablePorts" :key="port" :value="port">
                      {{ port }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p v-if="availablePorts.length === 0" class="text-xs text-destructive">
                  {{ t('messages.noSerialPortsHint') }}
                </p>
              </div>
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-2">
                  <Label>{{ t('connection.baudRate') }}</Label>
                  <Select v-model="tempConnectionConfig.rtuConfig.baudRate">
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem :value="9600">9600</SelectItem>
                      <SelectItem :value="19200">19200</SelectItem>
                      <SelectItem :value="38400">38400</SelectItem>
                      <SelectItem :value="115200">115200</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-2">
                  <Label>{{ t('connection.dataBits') }}</Label>
                  <Select v-model="tempConnectionConfig.rtuConfig.dataBits">
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem :value="7">7</SelectItem>
                      <SelectItem :value="8">8</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-2">
                  <Label>{{ t('connection.parity') }}</Label>
                  <Select :key="settingsStore.locale" v-model="tempConnectionConfig.rtuConfig.parity">
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="None">{{ t('connection.none') }}</SelectItem>
                      <SelectItem value="Even">{{ t('connection.even') }}</SelectItem>
                      <SelectItem value="Odd">{{ t('connection.odd') }}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-2">
                  <Label>{{ t('connection.stopBits') }}</Label>
                  <Select v-model="tempConnectionConfig.rtuConfig.stopBits">
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem :value="1">1</SelectItem>
                      <SelectItem :value="2">2</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </TabsContent>
          </Tabs>

          <DialogFooter>
            <Button variant="outline" @click="showConnectionDialog = false">{{ t('common.cancel') }}</Button>
            <Button @click="saveConnectionConfig" :disabled="!canSaveConnection">{{ t('common.save') }}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Write Value Dialog -->
      <Dialog v-model:open="showWriteDialog">
        <DialogContent class="sm:max-w-[320px]">
          <DialogHeader>
            <DialogTitle>{{ t('dialogs.write') }}</DialogTitle>
            <DialogDescription>
              {{ t('dialogs.writeAddress', { address: writeTarget.address }) }}
            </DialogDescription>
          </DialogHeader>
          <div class="grid gap-4 py-4">
            <div class="space-y-2">
              <Label>{{ t('data.current') }}: <span class="font-mono text-muted-foreground">{{ writeTarget.currentValue }}</span></Label>
              <Input v-model="writeTarget.newValue" type="text" class="font-mono text-lg" autofocus @keyup.enter="commitWrite" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" @click="showWriteDialog = false">{{ t('common.cancel') }}</Button>
            <Button @click="commitWrite">{{ t('actions.write') }} <Check class="w-4 h-4 ml-2"/></Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

    </div>
  </TooltipProvider>
</template>

<style>
/* No custom scrollbars needed */
</style>
