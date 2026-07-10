<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Plus, X, Terminal, Server, MonitorSmartphone, Settings2, Check } from '@lucide/vue'
import { ConnectMaster, DisconnectMaster, ReadRegisters, WriteRegister, WriteMultipleRegisters, StartSlave, StopSlave, GetSlaveData, UpdateSlaveData, ClearAllConnections, GetAvailablePorts } from '../wailsjs/go/main/App'
import { EventsOn, EventsOff } from '../wailsjs/runtime/runtime'
import { formatRegisterValue, parseUserInput } from './lib/modbusFormatter'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

// Types
import { useModbusStore, type ModbusInstance, createDefaultInstance } from './store/modbusStore'
import { storeToRefs } from 'pinia'

const modbusStore = useModbusStore()
const { instances, activeTab } = storeToRefs(modbusStore)

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
const showLogDialog = ref(false)

// Connection Setup temporary state
const tempConnectionConfig = ref<any>({})

// Available Serial Ports
const availablePorts = ref<string[]>([])
const fetchPorts = async () => {
  try {
    const ports = await GetAvailablePorts()
    availablePorts.value = ports || []
  } catch(e) {
    console.error("Failed to fetch ports", e)
  }
}

// System Status
const systemStatus = ref({ text: 'System Ready.', type: 'info' as 'info' | 'success' | 'error' })
const setStatus = (text: string, type: 'info' | 'success' | 'error' = 'info') => {
  systemStatus.value = { text, type }
}

const openConnectionDialog = async () => {
  if (!activeInstance.value) return
  await fetchPorts()
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
        setStatus(`[${inst.name}] Auto Read: ${inst.count} registers successfully.`, 'success')
        inst.status = 'connected' // Recovered!
        inst.hasError = false
      }
    } catch (e) {
      setStatus(`[${inst.name}] Auto Read error: ${e}. Reconnecting...`, 'error')
      inst.status = 'disconnected' // Reflect broken state in UI
      inst.hasError = true
      
      // Attempt to re-establish the socket for the next polling cycle
      try {
        await DisconnectMaster(inst.id)
        if (inst.protocol === 'tcp') {
          await ConnectMaster(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
        } else {
          await ConnectMaster(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
        }
      } catch (reconnectErr) {
        // Silent fail; next tick will just try again
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

const toggleAutoRead = (inst: ModbusInstance) => {
  inst.isAutoRead = !inst.isAutoRead
  if (inst.isAutoRead) {
    startPolling(inst)
    setStatus(`[${inst.name}] Auto Read started.`, 'info')
  } else {
    stopPolling(inst.id)
    setStatus(`[${inst.name}] Auto Read stopped.`, 'info')
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
          setStatus(`[${inst.name}] Read ${inst.count} registers successfully.`, 'success')
        }
      } else {
        const res = await GetSlaveData(inst.id, inst.startAddress, inst.count, inst.functionCode)
        if (res && res.length) {
          for(let i=0; i<res.length; i++) {
            inst.data[i] = res[i]
          }
          setStatus(`[${inst.name}] Memory refreshed.`, 'success')
        }
      }
    } catch (e) {
      setStatus(`[${inst.name}] Read error: ${e}`, 'error')
    }
}

const toggleConnection = async () => {
  if (!activeInstance.value) return
  const inst = activeInstance.value
  if (inst.status === 'connected') {
    try {
      if (inst.type === 'master') {
        await DisconnectMaster(inst.id)
      } else {
        await StopSlave(inst.id)
      }
      inst.status = 'disconnected'
      inst.hasError = false
      inst.isAutoRead = false
      inst.isAutoIncrement = false
      stopPolling(inst.id)
      setStatus(`[${inst.name}] Disconnected.`, 'info')
    } catch (e) {
      inst.hasError = true
      setStatus(`[${inst.name}] Disconnect failed: ${e}`, 'error')
    }
  } else {
    try {
      if (inst.type === 'master') {
        if (inst.protocol === 'tcp') {
          await ConnectMaster(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
          setStatus(`[${inst.name}] Connected to TCP ${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 'success')
        } else {
          await ConnectMaster(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
          setStatus(`[${inst.name}] Connected to RTU ${inst.rtuConfig.port}`, 'success')
        }
      } else {
        // SLAVE LOGIC
        if (inst.protocol === 'tcp') {
          await StartSlave(inst.id, 'tcp', `${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 0, 0, "", 0)
          setStatus(`[${inst.name}] Listening on TCP ${inst.tcpConfig.ip}:${inst.tcpConfig.port}`, 'success')
        } else {
          await StartSlave(inst.id, 'rtu', inst.rtuConfig.port, inst.rtuConfig.baudRate, inst.rtuConfig.dataBits, inst.rtuConfig.parity, inst.rtuConfig.stopBits)
          setStatus(`[${inst.name}] Listening on RTU ${inst.rtuConfig.port}`, 'success')
        }
      }
      inst.status = 'connected'
      inst.hasError = false
    } catch (e) {
      inst.hasError = true
      setStatus(`[${inst.name}] Connection failed: ${e}`, 'error')
    }
  }
}

// Write Dialog temporary state
const writeTarget = ref({ address: 0, currentValue: '' as string | number, newValue: '' as string | number })

const openWriteDialog = (address: number, currentValue: string | number) => {
  // Only allow writing if it's a writable function code and we're connected (simulated)
  if (activeInstance.value?.functionCode === '02' || activeInstance.value?.functionCode === '04') {
    alert("Function code 02 and 04 are Read-Only.")
    return
  }
  writeTarget.value = { address, currentValue, newValue: currentValue }
  showWriteDialog.value = true
}

const commitWrite = async () => {
  try {
    const inst = activeInstance.value!
    const parsedValues = parseUserInput(
      String(writeTarget.value.newValue),
      inst.dataType,
      inst.format,
      inst.byteOrder,
      inst.functionCode
    )
    
    if (inst.type === 'master') {
      const isCoil = inst.functionCode === '01'
      if (parsedValues.length === 1) {
        if (isCoil) {
          await (window as any).go.main.App.WriteCoil(inst.id, inst.slaveId, writeTarget.value.address, parsedValues[0])
        } else {
          await WriteRegister(inst.id, inst.slaveId, writeTarget.value.address, parsedValues[0])
        }
      } else {
        if (isCoil) {
          await (window as any).go.main.App.WriteMultipleCoils(inst.id, inst.slaveId, writeTarget.value.address, parsedValues)
        } else {
          await WriteMultipleRegisters(inst.id, inst.slaveId, writeTarget.value.address, parsedValues)
        }
      }
    } else {
      await UpdateSlaveData(inst.id, writeTarget.value.address, parsedValues, inst.functionCode)
      setStatus(`[${inst.name}] Updated local memory at ${writeTarget.value.address}.`, 'success')
    }
    readOnce(inst)
  } catch (e) {
    setStatus(`[${activeInstance.value!.name}] Write failed: ${e}`, 'error')
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
    setStatus(`[${inst.name}] Randomized ${inst.count} values based on ${inst.dataType}.`, 'success')
  } catch (e: any) {
    setStatus(`[${inst.name}] Randomize error: ${e}`, 'error')
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
    setStatus(`[${inst.name}] Increment error: ${e}`, 'error')
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
    setStatus(`[${inst.name}] Auto Increment started.`, 'info')
  } else {
    stopPolling(inst.id)
    setStatus(`[${inst.name}] Auto Increment stopped.`, 'info')
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
        setStatus(`[${inst.name}] External Write to addr ${addr}.`, 'success')
      }
    }
  })
})

onUnmounted(() => {
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

const addInstance = (type: 'master' | 'slave') => {
  const maxId = instances.value.reduce((max, inst) => Math.max(max, parseInt(inst.id) || 0), 0)
  const newId = String(maxId + 1)
  const name = newInstanceName.value.trim() || `${type === 'master' ? 'Master' : 'Slave'} ${newId}`

  if (instances.value.some(inst => inst.name.toLowerCase() === name.toLowerCase())) {
    nameError.value = 'A connection with this name already exists.'
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

// Generate Mock Data for 11-column matrix
const getMatrixRows = (instance: ModbusInstance) => {
  const rowCount = Math.ceil(instance.count / 10)
  const isCoil = instance.functionCode === '01' || instance.functionCode === '02'
  const is32Bit = !isCoil && (instance.dataType === 'Int32' || instance.dataType === 'UInt32' || instance.dataType === 'Float32')
  
  return Array.from({ length: rowCount }).map((_, rIndex) => {
    return Array.from({ length: 10 }).map((_, cIndex) => {
      const addr = rIndex * 10 + cIndex
      let val = null
      if (addr < instance.count) {
        if (is32Bit && addr % 2 !== 0) {
          val = '-'
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
        rawValue: rawVal
      }
    })
  })
}

// Global Logs Mock Data
const globalLogs = ref([
  { time: '10:45:01.120', type: 'TX', target: 'Master 1', data: '00 01 00 00 00 06 01 03 00 00 00 0A' },
  { time: '10:45:01.145', type: 'RX', target: 'Master 1', data: '00 01 00 00 00 17 01 03 14 00 00 00 00 ...' },
])
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
                    title="Close Connection"
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
                    <p>Add Connection Instance</p>
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
                <span class="opacity-80">{{ instance.type === 'master' ? 'Master' : 'Slave' }}</span>
                <span class="text-muted-foreground font-normal">•</span> 
                <span class="font-mono text-[13px] text-foreground">{{ getConnectionInfoText(instance) }}</span>
              </h2>

              <Button 
                variant="outline"
                size="sm" 
                class="min-w-[110px] transition-colors ml-2"
                @click="toggleConnection"
              >
                {{ instance.status === 'connected' ? 'Disconnect' : 'Connect' }}
              </Button>

              <Button variant="outline" size="sm" class="min-w-[90px]" @click="openConnectionDialog" :disabled="instance.status === 'connected'">
                <Settings2 class="w-4 h-4 mr-2 text-muted-foreground" /> Setup
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
                Randomize
              </Button>

              <Button 
                v-if="instance.type === 'slave'"
                variant="outline"
                size="sm"
                class="min-w-[130px] transition-colors" 
                @click="toggleAutoIncrement(instance)"
                :disabled="instance.status !== 'connected'"
              >
                {{ instance.isAutoIncrement ? 'Stop Auto Incr' : 'Auto Increment' }}
              </Button>


              <Button 
                v-if="instance.type === 'master'"
                variant="outline"
                size="sm"
                class="min-w-[130px] transition-colors" 
                @click="toggleAutoRead(instance)"
                :disabled="instance.status !== 'connected'"
              >
                {{ instance.isAutoRead ? 'Stop Auto Read' : 'Auto Read' }}
              </Button>
              
              <Button 
                v-if="instance.type === 'master'"
                variant="outline"
                size="sm"
                class="min-w-[100px]"
                :disabled="instance.status !== 'connected' || instance.isAutoRead"
                @click="readOnce(instance)"
              >
                Read Once
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
                  <Label class="text-[11px] font-semibold text-muted-foreground">Unit ID</Label>
                  <Input v-model="instance.slaveId" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" />
                </div>
                <div class="space-y-1.5 w-[140px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Function</Label>
                  <Select v-model="instance.functionCode" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @update:modelValue="onAddressingChange(instance)">
                    <SelectTrigger class="h-8 text-xs bg-background shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs">
                      <SelectItem value="01">Coils (0x01)</SelectItem>
                      <SelectItem value="02">Discrete (0x02)</SelectItem>
                      <SelectItem value="03">Holding (0x03)</SelectItem>
                      <SelectItem value="04">Input (0x04)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-1.5 w-[80px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Start Addr</Label>
                  <Input v-model="instance.startAddress" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @change="onAddressingChange(instance)" />
                </div>
                <div class="space-y-1.5 w-[70px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Count</Label>
                  <Input v-model="instance.count" type="number" max="100" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" @change="onAddressingChange(instance)" />
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 2: Parsing -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[100px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">Data Type</Label>
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
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">Format</Label>
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
                  <Label class="text-[11px] font-semibold text-muted-foreground" :class="{ 'opacity-50': instance.functionCode === '01' || instance.functionCode === '02' }">Byte Order</Label>
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
                  <Label class="text-[11px] font-semibold text-muted-foreground">Interval(ms)</Label>
                  <Input v-model="instance.intervalMs" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" :disabled="instance.type === 'master' ? instance.isAutoRead : instance.isAutoIncrement" />
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 4: Display Options -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 flex flex-col items-center justify-center">
                  <Label class="text-[11px] font-semibold text-muted-foreground cursor-pointer" @click="modbusStore.toggleRawData()">Raw</Label>
                  <div class="h-8 flex items-center">
                    <button 
                      @click="modbusStore.toggleRawData()" 
                      type="button" 
                      class="relative inline-flex h-4 w-7 shrink-0 cursor-pointer items-center justify-center rounded-full border-2 border-transparent focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background transition-colors"
                      :class="modbusStore.showRawData ? 'bg-primary' : 'bg-input'"
                    >
                      <span class="sr-only">Toggle Raw Data</span>
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
                        <TableHead class="w-24 text-center border-b border-r border-border font-bold text-foreground">Address</TableHead>
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
                          <Tooltip v-if="cell.value !== null">
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
                              <p>Click to write to address <span class="font-mono text-primary">{{ instance.startAddress + rIdx * 10 + cIdx }}</span></p>
                            </TooltipContent>
                          </Tooltip>
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
            {{ systemStatus.text }}
          </span>
        </div>
        
        <Button variant="ghost" size="sm" @click="showLogDialog = true" class="h-7 text-xs text-muted-foreground hover:text-foreground bg-muted/50 hover:bg-muted ring-1 ring-border/50 shadow-sm px-3 rounded-lg">
          <Terminal class="w-3.5 h-3.5 mr-1.5" />
          Global Log Console
        </Button>
      </footer>

      <!-- Add Instance Dialog -->
      <Dialog v-model:open="showAddDialog">
        <DialogContent class="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add New Connection</DialogTitle>
            <DialogDescription>
              Choose the type of Modbus instance you want to create.
            </DialogDescription>
          </DialogHeader>
          <div class="grid gap-4 py-4">
            <div class="space-y-2 mb-2">
              <Label :class="{ 'text-destructive': nameError }">Connection Name</Label>
              <Input 
                v-model="newInstanceName" 
                placeholder="e.g. Pump Station 1 (Optional)" 
                autofocus 
                @keyup.enter="addInstance('master')" 
                :class="{ 'border-destructive focus-visible:ring-destructive': nameError }"
              />
              <p v-if="nameError" class="text-[11px] font-medium text-destructive mt-1">{{ nameError }}</p>
            </div>
            
            <Label class="text-muted-foreground">Select Role to Create</Label>
            <div class="grid grid-cols-2 gap-4">
              <Button 
                variant="outline" 
                class="h-24 flex flex-col items-center justify-center gap-2 hover:border-primary hover:text-primary transition-colors"
                @click="addInstance('master')"
              >
                <MonitorSmartphone class="w-8 h-8" />
                <span class="font-semibold">Master (Client)</span>
              </Button>
              <Button 
                variant="outline" 
                class="h-24 flex flex-col items-center justify-center gap-2 hover:border-primary hover:text-primary transition-colors"
                @click="addInstance('slave')"
              >
                <Server class="w-8 h-8" />
                <span class="font-semibold">Slave (Server)</span>
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <!-- Connection Setup Dialog -->
      <Dialog v-model:open="showConnectionDialog">
        <DialogContent class="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>Connection Setup</DialogTitle>
            <DialogDescription>
              Configure network or serial parameters for this instance.
            </DialogDescription>
          </DialogHeader>
          
          <Tabs v-model="tempConnectionConfig.protocol" class="mt-2">
            <TabsList class="grid w-full grid-cols-2">
              <TabsTrigger value="tcp">Modbus TCP</TabsTrigger>
              <TabsTrigger value="rtu">Modbus RTU</TabsTrigger>
            </TabsList>
            
            <!-- TCP Config -->
            <TabsContent value="tcp" class="space-y-4 pt-4">
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">IP Address</Label>
                <Input v-model="tempConnectionConfig.tcpConfig.ip" class="col-span-3 font-mono" />
              </div>
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">Port</Label>
                <Input v-model="tempConnectionConfig.tcpConfig.port" type="number" class="col-span-3 font-mono" />
              </div>
            </TabsContent>
            
            <!-- RTU Config -->
            <TabsContent value="rtu" class="space-y-4 pt-4">
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">Port Name</Label>
                <div class="col-span-3">
                  <Input v-model="tempConnectionConfig.rtuConfig.port" placeholder="COM1 or /dev/ttyS0" class="font-mono" list="serial-ports" />
                  <datalist id="serial-ports">
                    <option v-for="port in availablePorts" :key="port" :value="port" />
                  </datalist>
                </div>
              </div>
              <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">Baud Rate</Label>
                <Select v-model="tempConnectionConfig.rtuConfig.baudRate">
                  <SelectTrigger class="col-span-3"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem :value="9600">9600</SelectItem>
                    <SelectItem :value="19200">19200</SelectItem>
                    <SelectItem :value="38400">38400</SelectItem>
                    <SelectItem :value="115200">115200</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </TabsContent>
          </Tabs>

          <DialogFooter class="mt-4">
            <Button variant="outline" @click="showConnectionDialog = false">Cancel</Button>
            <Button @click="saveConnectionConfig">Save Changes</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Write Value Dialog -->
      <Dialog v-model:open="showWriteDialog">
        <DialogContent class="sm:max-w-[320px]">
          <DialogHeader>
            <DialogTitle>Write Register</DialogTitle>
            <DialogDescription>
              Address: <span class="font-mono font-bold text-primary">{{ writeTarget.address }}</span>
            </DialogDescription>
          </DialogHeader>
          <div class="grid gap-4 py-4">
            <div class="space-y-2">
              <Label>Current Value: <span class="font-mono text-muted-foreground">{{ writeTarget.currentValue }}</span></Label>
              <Input v-model="writeTarget.newValue" type="number" step="any" class="font-mono text-lg" autofocus @keyup.enter="commitWrite" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" @click="showWriteDialog = false">Cancel</Button>
            <Button @click="commitWrite">Write <Check class="w-4 h-4 ml-2"/></Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Global Log Dialog -->
      <Dialog v-model:open="showLogDialog">
        <DialogContent class="sm:max-w-[700px]">
          <DialogHeader>
            <DialogTitle>Global Log Console</DialogTitle>
            <DialogDescription>
              Raw Modbus traffic for all instances.
            </DialogDescription>
          </DialogHeader>
          
          <ScrollArea class="h-[400px] w-full rounded-md border bg-black text-emerald-400 p-4 font-mono text-xs">
            <div v-for="(log, i) in globalLogs" :key="i" class="mb-2 flex gap-3">
              <span class="text-zinc-500 shrink-0">[{{ log.time }}]</span>
              <span :class="log.type === 'TX' ? 'text-blue-400' : 'text-emerald-400'" class="font-bold shrink-0">{{ log.type }}</span>
              <span class="text-zinc-400 shrink-0 w-16 truncate">{{ log.target }}</span>
              <span class="text-zinc-300 break-all">{{ log.data }}</span>
            </div>
            <ScrollBar orientation="vertical" />
          </ScrollArea>
        </DialogContent>
      </Dialog>
    </div>
  </TooltipProvider>
</template>

<style>
/* No custom scrollbars needed */
</style>
