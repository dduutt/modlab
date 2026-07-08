<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Plus, X, Terminal, Play, Square, Activity, Server, MonitorSmartphone, Settings2, Check } from '@lucide/vue'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Card } from '@/components/ui/card'

// Types
interface ModbusInstance {
  id: string
  name: string
  type: 'master' | 'slave'
  status: 'connected' | 'disconnected'
  protocol: 'tcp' | 'rtu'
  tcpConfig: { ip: string, port: number }
  rtuConfig: { port: string, baudRate: number, dataBits: number, stopBits: number, parity: string }
  slaveId: number
  functionCode: string
  startAddress: number
  count: number
  dataType: string
  format: string
  byteOrder: string
  intervalMs: number
  isAutoRead: boolean
}

const createDefaultInstance = (id: string, type: 'master' | 'slave'): ModbusInstance => ({
  id,
  name: `${type === 'master' ? 'Master' : 'Slave'} ${id}`,
  type,
  status: 'disconnected',
  protocol: 'tcp',
  tcpConfig: { ip: '127.0.0.1', port: 502 },
  rtuConfig: { port: 'COM1', baudRate: 9600, dataBits: 8, stopBits: 1, parity: 'None' },
  slaveId: 1,
  functionCode: '03',
  startAddress: 0,
  count: 40,
  dataType: 'Int16',
  format: 'Dec',
  byteOrder: 'ABCD',
  intervalMs: 1000,
  isAutoRead: false,
})

const instances = ref<ModbusInstance[]>([
  createDefaultInstance('1', 'master')
])

const activeTab = ref('1')
let nextId = 2

const activeInstance = computed(() => instances.value.find(i => i.id === activeTab.value))

// Dynamic connection info badge
const connectionInfoText = computed(() => {
  if (!activeInstance.value) return 'Not Configured'
  const inst = activeInstance.value
  if (inst.protocol === 'tcp') return `TCP ${inst.tcpConfig.ip}:${inst.tcpConfig.port}`
  return `RTU ${inst.rtuConfig.port} @ ${inst.rtuConfig.baudRate}`
})

// Dialog States
const showAddDialog = ref(false)
const showConnectionDialog = ref(false)
const showWriteDialog = ref(false)
const showLogDialog = ref(false)

// Connection Setup temporary state
const tempConnectionConfig = ref<any>({})

const openConnectionDialog = () => {
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
  activeInstance.value.protocol = tempConnectionConfig.value.protocol
  activeInstance.value.tcpConfig = tempConnectionConfig.value.tcpConfig
  activeInstance.value.rtuConfig = tempConnectionConfig.value.rtuConfig
  showConnectionDialog.value = false
}

const toggleConnection = () => {
  if (!activeInstance.value) return
  if (activeInstance.value.status === 'connected') {
    activeInstance.value.status = 'disconnected'
    activeInstance.value.isAutoRead = false
  } else {
    activeInstance.value.status = 'connected'
  }
}

// Write Dialog temporary state
const writeTarget = ref({ address: 0, currentValue: 0, newValue: 0 })

const openWriteDialog = (address: number, currentValue: number) => {
  // Only allow writing if it's a writable function code and we're connected (simulated)
  if (activeInstance.value?.functionCode === '02' || activeInstance.value?.functionCode === '04') {
    alert("Function code 02 and 04 are Read-Only.")
    return
  }
  writeTarget.value = { address, currentValue, newValue: currentValue }
  showWriteDialog.value = true
}

const commitWrite = () => {
  // In real app, call Wails backend here
  console.log(`Writing ${writeTarget.value.newValue} to address ${writeTarget.value.address}`)
  showWriteDialog.value = false
}

// Instance Management
const addInstance = (type: 'master' | 'slave') => {
  const newId = String(nextId++)
  instances.value.push(createDefaultInstance(newId, type))
  activeTab.value = newId
  showAddDialog.value = false
}

const removeInstance = (id: string, event: Event) => {
  event.stopPropagation()
  if (instances.value.length === 1) return
  const index = instances.value.findIndex(i => i.id === id)
  instances.value.splice(index, 1)
  if (activeTab.value === id) {
    activeTab.value = instances.value[Math.max(0, index - 1)]?.id || instances.value[0].id
  }
}

// Generate Mock Data for 11-column matrix
const getMatrixRows = (instance: ModbusInstance) => {
  const rowCount = Math.ceil(instance.count / 10)
  return Array.from({ length: rowCount }).map((_, rIndex) => {
    return Array.from({ length: 10 }).map((_, cIndex) => {
      const addr = rIndex * 10 + cIndex
      return {
        address: addr,
        value: addr < instance.count ? 0 : null // null for out of bounds
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
    <div class="h-screen flex flex-col bg-muted/20 text-foreground font-sans overflow-hidden select-none relative">
      
      <!-- Native shadcn Tabs Component managing the whole app view -->
      <Tabs v-model="activeTab" class="flex-1 flex flex-col overflow-hidden relative z-10">
        
        <!-- Row 1: Top Tab View -->
        <div class="bg-card/80 backdrop-blur-xl border-b border-border/50 flex items-end pl-3 pr-4 pt-3 shrink-0">
          <div class="w-full overflow-x-auto no-scrollbar flex items-end">
            <TabsList class="bg-transparent h-auto p-0 flex items-end justify-start gap-1.5 rounded-none border-b-0">
              <TabsTrigger 
                v-for="instance in instances" :key="instance.id" :value="instance.id"
                class="group flex items-center gap-2 px-5 py-2.5 text-sm transition-all duration-300 border border-b-0 rounded-t-xl outline-none data-[state=active]:bg-card/60 data-[state=active]:backdrop-blur-md data-[state=active]:text-foreground data-[state=active]:border-border/50 data-[state=active]:shadow-sm data-[state=active]:z-10 data-[state=inactive]:bg-transparent data-[state=inactive]:border-transparent data-[state=inactive]:text-muted-foreground hover:data-[state=inactive]:bg-muted/50 cursor-pointer"
              >
                <MonitorSmartphone v-if="instance.type === 'master'" class="w-4 h-4 shrink-0" :class="activeTab === instance.id ? 'text-primary' : 'text-muted-foreground'" />
                <Server v-else class="w-4 h-4 shrink-0" :class="activeTab === instance.id ? 'text-primary' : 'text-muted-foreground'" />
                
                <span class="font-medium tracking-wide max-w-[140px] truncate select-none">{{ instance.name }}</span>
                
                <div 
                  v-if="instances.length > 1"
                  @click.stop="removeInstance(instance.id, $event)"
                  class="w-5 h-5 ml-1 shrink-0 rounded-md flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition-all"
                  title="Close Connection"
                >
                  <X class="w-3.5 h-3.5" />
                </div>
                <div v-else class="w-5 h-5 ml-1 shrink-0"></div>
              </TabsTrigger>

              <!-- Add Button sitting natively as a pseudo-tab right next to the last tab -->
              <div class="flex items-center justify-center h-full pb-1 pl-1 pr-4">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" @click="showAddDialog = true" class="h-8 w-8 text-muted-foreground hover:text-foreground shrink-0">
                      <Plus class="w-4 h-4" />
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

        <!-- Render active tab content natively through shadcn TabsContent -->
        <TabsContent 
          v-for="instance in instances" 
          :key="instance.id" 
          :value="instance.id" 
          class="flex-1 flex flex-col min-h-0 m-0 focus-visible:outline-none"
        >
          
          <!-- Row 2: Master 配置信息一行 (Master Configuration) -->
          <div class="px-5 py-3 border-b border-border/50 bg-card/60 backdrop-blur-md flex flex-wrap items-center justify-between gap-4 shrink-0 shadow-[0_4px_20px_-10px_rgba(0,0,0,0.05)] z-20">
            <div class="flex items-center gap-3 shrink-0">
              <h2 class="text-sm font-semibold flex items-center gap-2">
                <span class="opacity-80">{{ instance.type === 'master' ? 'Master' : 'Slave' }}</span>
                <span class="text-muted-foreground font-normal">•</span> 
                
                <button 
                  @click="toggleConnection"
                  class="flex items-center gap-1.5 font-mono text-[13px] px-1.5 py-0.5 rounded transition-colors group"
                  :class="instance.status === 'connected' ? 'text-foreground hover:bg-emerald-500/10' : 'text-muted-foreground hover:bg-muted'"
                  title="Click to toggle connection"
                >
                  <div 
                    class="h-2 w-2 rounded-full transition-colors" 
                    :class="instance.status === 'connected' ? 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]' : 'bg-muted-foreground/40'"
                  ></div>
                  <span>{{ instance.status === 'connected' ? connectionInfoText : 'Disconnected' }}</span>
                </button>
              </h2>
              <Button variant="outline" size="sm" class="h-7 text-xs bg-background shadow-sm shrink-0" @click="openConnectionDialog">
                <Settings2 class="w-3.5 h-3.5 mr-1.5" /> Setup Connection...
              </Button>
            </div>

            <!-- Right Side: Read Action Buttons (Moved to Master Row) -->
            <div class="flex items-center gap-3 shrink-0 ml-auto">
              <Button 
                class="h-8 transition-all px-4 font-semibold text-xs shadow-sm shrink-0" 
                :variant="instance.isAutoRead ? 'destructive' : 'outline'"
                @click="instance.isAutoRead = !instance.isAutoRead"
                :disabled="instance.status !== 'connected'"
              >
                <Square v-if="instance.isAutoRead" class="w-3.5 h-3.5 mr-1.5 fill-current shrink-0" />
                <Play v-else class="w-3.5 h-3.5 mr-1.5 fill-current shrink-0" />
                {{ instance.isAutoRead ? 'Stop Auto Read' : 'Auto Read' }}
              </Button>
              
              <Button 
                variant="outline"
                class="h-8 font-semibold px-6 text-xs shadow-sm shrink-0"
                :disabled="instance.status !== 'connected'"
              >
                Read Once
              </Button>
            </div>
          </div>

          <!-- Row 3: 采集配置一行 (Collection Configuration) -->
          <div class="px-5 py-4 border-b border-border/50 bg-card/40 backdrop-blur-sm flex flex-wrap items-center justify-between gap-4 shadow-sm z-10 shrink-0">
            
            <!-- Left Side: Parameter Groups -->
            <div class="flex flex-wrap items-center gap-y-4 gap-x-8">
              
              <!-- Group 1: Addressing -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[70px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Unit ID</Label>
                  <Input v-model="instance.slaveId" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" />
                </div>
                <div class="space-y-1.5 w-[140px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Function</Label>
                  <Select v-model="instance.functionCode">
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
                  <Input v-model="instance.startAddress" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" />
                </div>
                <div class="space-y-1.5 w-[70px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Count</Label>
                  <Input v-model="instance.count" type="number" max="100" class="h-8 text-xs font-mono bg-background shadow-sm" />
                </div>
              </div>

              <div class="w-px h-8 bg-border/50 hidden md:block"></div>

              <!-- Group 2: Parsing -->
              <div class="flex items-center gap-3 shrink-0">
                <div class="space-y-1.5 w-[100px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Data Type</Label>
                  <Select v-model="instance.dataType">
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
                  <Label class="text-[11px] font-semibold text-muted-foreground">Format</Label>
                  <Select v-model="instance.format">
                    <SelectTrigger class="h-8 text-xs bg-background shadow-sm"><SelectValue /></SelectTrigger>
                    <SelectContent class="text-xs">
                      <SelectItem value="Dec">Dec</SelectItem>
                      <SelectItem value="Hex">Hex</SelectItem>
                      <SelectItem value="Bin">Bin</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-1.5 w-[90px]">
                  <Label class="text-[11px] font-semibold text-muted-foreground">Byte Order</Label>
                  <Select v-model="instance.byteOrder">
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
                  <Input v-model="instance.intervalMs" type="number" class="h-8 text-xs font-mono bg-background shadow-sm" />
                </div>
              </div>

            </div>
          </div>

          <!-- Row 4: Data Matrix View (数据区域一行) -->
          <div class="flex-1 p-4 md:p-6 min-h-0 overflow-hidden">
            <ScrollArea class="h-full w-full rounded-2xl ring-1 ring-foreground/10 bg-card/90 backdrop-blur-2xl shadow-xl">
              <Table class="w-full text-sm">
                <TableHeader class="bg-muted/30 sticky top-0 z-10 border-b border-border/50 backdrop-blur-md">
                  <TableRow class="hover:bg-transparent border-none">
                    <TableHead class="w-24 text-center border-r font-bold text-foreground">Address</TableHead>
                    <TableHead v-for="i in 10" :key="i" class="text-center font-bold text-foreground w-[9%]">
                      +{{ i - 1 }}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow 
                    v-for="(row, rIdx) in getMatrixRows(instance)" 
                    :key="rIdx"
                    class="transition-colors border-border/50"
                    :class="{ 'bg-muted/20': rIdx % 2 !== 0 }"
                  >
                    <!-- Base Address Column -->
                    <TableCell class="font-mono text-primary font-semibold text-center border-r bg-muted/10">
                      {{ instance.startAddress + rIdx * 10 }}
                    </TableCell>
                    
                    <!-- Data Cells -->
                    <TableCell v-for="(cell, cIdx) in row" :key="cIdx" class="p-1">
                      <Tooltip v-if="cell.value !== null">
                        <TooltipTrigger asChild>
                          <button 
                            @click="openWriteDialog(instance.startAddress + rIdx * 10 + cIdx, cell.value)"
                            class="w-full h-8 font-mono text-center rounded bg-transparent hover:bg-muted focus:bg-primary/10 focus:text-primary focus:ring-1 focus:ring-primary/50 transition-all text-foreground"
                          >
                            {{ cell.value }}
                          </button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>Click to write to address <span class="font-mono text-primary">{{ instance.startAddress + rIdx * 10 + cIdx }}</span></p>
                        </TooltipContent>
                      </Tooltip>
                      <div v-else class="w-full h-8 bg-transparent"></div>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <ScrollBar orientation="horizontal" />
            </ScrollArea>
          </div>
        </TabsContent>
        
      </Tabs>

      <!-- Bottom Status Bar -->
      <footer class="h-10 bg-card/80 backdrop-blur-xl border-t border-border/50 flex items-center justify-between px-4 z-30 shrink-0">
        <div class="flex items-center gap-4 text-xs">
          <span class="text-muted-foreground font-medium flex items-center gap-2">
            <div class="h-1.5 w-1.5 rounded-full bg-primary/50 animate-pulse"></div>
            System Ready.
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
          <div class="grid grid-cols-2 gap-4 py-4">
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
                <Input v-model="tempConnectionConfig.rtuConfig.port" placeholder="COM1 or /dev/ttyS0" class="col-span-3 font-mono" />
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
              <Input v-model="writeTarget.newValue" type="number" class="font-mono text-lg" autofocus @keyup.enter="commitWrite" />
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
