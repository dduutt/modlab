import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ModbusInstance {
  id: string
  name: string
  type: 'master' | 'slave'
  protocol: 'tcp' | 'rtu'
  tcpConfig: { ip: string, port: number }
  rtuConfig: { port: string, baudRate: number, dataBits: number, parity: string, stopBits: number }
  slaveId: number
  functionCode: string
  startAddress: number
  count: number
  dataType: string
  format: string
  byteOrder: string
  intervalMs: number
  status: 'connected' | 'disconnected'
  hasError: boolean
  isAutoRead: boolean
  isAutoIncrement: boolean
  data: number[]
}

export const createDefaultInstance = (id: string, type: 'master' | 'slave'): ModbusInstance => ({
  id,
  name: `${type === 'master' ? 'Master' : 'Slave'} ${id}`,
  type,
  status: 'disconnected',
  hasError: false,
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
  isAutoIncrement: false,
  data: Array(100).fill(0),
})

export const useModbusStore = defineStore('modbus', () => {
  const instances = ref<ModbusInstance[]>([createDefaultInstance('1', 'master')])
  const activeTab = ref<string>('1')
  const showRawData = ref<boolean>(false)
  const toggleRawData = () => { showRawData.value = !showRawData.value }

  return { instances, activeTab, showRawData, toggleRawData }
}, {
  persist: {
    key: 'modbus_store',
    serializer: {
      serialize: (state: any) => {
        const toSave = {
          activeTab: state.activeTab,
          instances: state.instances.map((i: any) => ({ ...i, data: [] }))
        }
        return JSON.stringify(toSave)
      },
      deserialize: (str: string) => {
        const parsed = JSON.parse(str)
        if (parsed && parsed.instances && parsed.instances.length > 0) {
          parsed.instances.forEach((inst: any) => {
            inst.status = 'disconnected'
            inst.isAutoRead = false
            inst.hasError = false
            inst.data = new Array(inst.count || 100).fill(0)
          })
          return parsed
        }
        return {
          instances: [createDefaultInstance('1', 'master')],
          activeTab: '1'
        }
      }
    }
  }
})
