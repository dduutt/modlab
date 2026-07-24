# Tauri 2.0 原生多窗口 Modbus 报文监控系统设计规范

## 概述

本规范文档详细定义了 Modbus 桌面应用（Tauri 2.0 + Vue 3）原生多窗口报文监控系统的设计方案。系统通过 Tauri 2.0 `WebviewWindow` 创建独立的原生窗口，利用全局事件总线（Event Bus）进行跨窗口实时报文广播，并提供设备唯一性校验、设备筛选、TX/RX 方向过滤、窗口置顶与自动追随滚动。

---

## 1. 设备名称唯一性校验 (Device Name Uniqueness Validation)

- **唯一性约束**：所有会话设备的名称 (`title`) 在系统中必须保持唯一（如 `Slave 1`, `Slave 2`, `Master 1`），禁止重名。
- **新建设备校验 (`NewDeviceModal.vue`)**：
  - 点击“创建设备”时，校验输入的名称是否已存在于当前打开的会话列表中；
  - 若重名，在弹窗中显示红色提示（`设备名称 "Slave 1" 已存在，请使用其他名称`），阻止创建。
- **双击重命名校验 (`TabBar.vue`)**：
  - 双击标签页重命名失焦或回车时，校验新名称是否与其它标签页重复；若重复则取消修改并提示。

---

## 2. 窗口生命周期管理 (`windowService.ts`)

- **唤起入口**：在主界面顶栏 Toolbar 与底栏 StatusBar 上均提供 `Traffic Log` 按钮。
- **窗口实例化**：
  - 点击按钮通过 `@tauri-apps/api/webviewWindow` 的 `WebviewWindow` 创建标识为 `traffic-log-window` 的窗口；
  - 窗口属性：
    - **URL**: `/#/traffic-log`
    - **Title**: `Modbus Communication Traffic Log - Modlab`
    - **Dimensions**: 默认宽度 `850px`，默认高度 `550px`
    - **Resizable**: `true`
    - **AlwaysOnTop**: 可在界面上动态切换
- **防重复打开**：调用 `WebviewWindow.getByLabel('traffic-log-window')`，若窗口已存在，直接调用 `window.focus()` 唤醒置前，不重复打开新窗口。

---

## 3. 跨窗口实时通信总线 (`@tauri-apps/api/event`)

### A. 事件发布 (Main Window -> Traffic Window)
- 主窗口在发起 TX 请求或收到 RX 响应时，调用 Tauri `emit('traffic-log-entry', payload)` 广播全套数据：
  ```typescript
  interface TrafficLogPayload {
    id: string;
    sessionId: string;
    sessionTitle: string;
    direction: 'TX' | 'RX';
    message: string;
    bytes: string;
    timestamp: string;
  }
  ```

### B. 事件监听 (Traffic Window)
- 报文独立窗口在组件挂载时调用 `listen<TrafficLogPayload>('traffic-log-entry', callback)`，无感追加新数据；
- 在窗口卸载时解绑监听，防止内存泄露。

---

## 4. 独立报文窗口界面与高级筛选 (`/#/traffic-log`)

### A. 控制栏功能 (Toolbar Control Bar)
- **设备筛选下拉框 (Device Selector)**：
  - 动态收集出现的设备列表，提供选项 `All Devices`（默认）及各个独立设备（如 `Slave 1`, `Master 1`）；
- **方向筛选切页 (Direction Filter)**：
  - `ALL`：展示所有方向报文；
  - `TX Only`：仅展示发送报文（蓝色 Tag 标注）；
  - `RX Only`：仅展示接收报文（绿色 Tag 标注）。
- **操作控制**：
  - 🧹 **Clear Log**：一键清空记录；
  - 📌 **Always on Top**：切换窗口系统级置顶标志；
  - 🔽 **Auto Scroll**：开启时处于底部自动随新报文下滚，向上手动滚动时自动暂缓跟进。

### B. 报文日志条目展示 (Log Item View)
- 包含字段：`[时间戳] [设备名称 Badge] [TX/RX Badge] [请求摘要] [Hex 原始字节串]`；
- 鼠标悬浮某条报文时，右侧提供 **复制 Hex** 按钮。

---

## 5. 显式排除的特性 (Non-Requirements)

- 不使用传统 HTML Modal 弹窗（全量使用 Tauri 原生 Multi-Window）；
- 不做底层二进制解包校验分析，以清晰可视的 Hex 字节流与 Modbus 请求摘要为主。
