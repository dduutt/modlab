# Modlab

Modlab 是一款基于 Wails 构建的跨平台 Modbus 调试与模拟工具，支持作为主机（Client）轮询设备，或作为从机（Server）模拟数据响应。

## 特性

- **协议支持**：Modbus TCP、Modbus RTU (串口)。
- **工作模式**：
  - **主机模式**：支持按指定频率自动读取、单次读取，支持修改远程设备数据。
  - **从机模拟**：支持本地开启 TCP/RTU 服务，支持自动累加数据以便模拟传感器流。
- **数据管理**：
  - 支持线圈 (01)、离散输入 (02)、保持寄存器 (03)、输入寄存器 (04)。
  - 四个数据区各自独立，符合标准 Modbus 规范，对 02 和 04 区域实施严格的写保护。
- **数据格式解析**：
  - 支持 Int16、UInt16、Int32、UInt32、Float32。
  - 支持字节序 (Byte Order) 切换：ABCD、CDAB、DCBA、BADC。
  - 支持格式化显示：十进制、十六进制、二进制及原始 Hex 数据。



## 功能码支持

| 功能码 | 描述 |
| :---: | --- |
| 0x01 | Read Coils (读线圈) |
| 0x02 | Read Discrete Inputs (读离散输入) |
| 0x03 | Read Holding Registers (读保持寄存器) |
| 0x04 | Read Input Registers (读输入寄存器) |
| 0x05 | Write Single Coil (写单个线圈) |
| 0x06 | Write Single Register (写单个寄存器) |
| 0x0F | Write Multiple Coils (写多个线圈) |
| 0x10 | Write Multiple Registers (写多个寄存器) |

## 技术栈

- 前端：Vue 3 + Vite + Tailwind CSS + Pinia
- 后端：Go + Wails v2
- 依赖库：`github.com/dduutt/modbus`、`go.bug.st/serial`
