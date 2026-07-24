# 工业级 Modbus 数据与通信功能增强规范文档

## 概述

本规范文档旨在提升 Modbus 桌面应用（Tauri + Vue 3）作为专业工业调试工具的功能完善度，涵盖 32 位高级数据解析（Float32 / Int32 及 4 种字节序翻转）、CSV 点位表导入/导出、以及通信超时与重试参数控制。

---

## 1. 32 位高级数据解析与字节序翻转 (Float32 / Int32 & Endianness)

### A. 2 寄存器自动跨列合并显示
- **功能码约束**：仅在 `0x03` (Holding) 和 `0x04` (Input Registers) 下生效。
- **数据类型切换**：
  - **`Int16` / `UInt16`**：网格每单元格表示 1 个 16 位寄存器（Address 0, 1, 2, 3...）。
  - **`Float32` / `Int32` / `UInt32`**：网格自动按每 2 个连续寄存器（如 Address 0 [0-1], 2 [2-3], 4 [4-5]...）合并显示为一个 32 位数据单元。

### B. 4 种工业级字节序 (Byte Order) 编解码规则
对跨 2 个寄存器（4 字节 A B C D）进行位翻转解析：
- **`ABCD`**（标准大端 Big-Endian）：`[Byte 1, Byte 2, Byte 3, Byte 4]`
- **`CDAB`**（小端字节交换 Little-Endian）：`[Byte 3, Byte 4, Byte 1, Byte 2]`
- **`BADC`**（字交换 Word Swap）：`[Byte 2, Byte 1, Byte 4, Byte 3]`
- **`DCBA`**（大端字节交换 Big-Endian Byte Swap）：`[Byte 4, Byte 3, Byte 2, Byte 1]`

### C. 32 位内联编辑与编码反写
- 双击 `Float32` / `Int32` 单元格时，弹出输入框允许直接输入带小数点的数值（如 `12.34`）或 32 位整数；
- 敲击回车或失焦保存时，系统根据当前选择的 `Byte Order` 规则将 32 位数值拆分为 2 个 16 位 Word，并依次保存至对应的 2 个寄存器内存点位。

---

## 2. CSV 点位表导入与导出 (CSV Import & Export)

### A. 导出 CSV (Export CSV)
- 点击 Toolbar 的 **Export** 按钮，弹出文件保存对话框（或直接触发浏览器/Tauri 文件下载）；
- 导出文件格式为 `.csv`，包含表头：
  ```csv
  Address,Value
  0,1234
  1,5678
  ```
- 方便工程人员在 Excel / 文本编辑器中直接查看与编辑点位映射表。

### B. 导入 CSV (Import CSV)
- 点击 Toolbar 的 **Import** 按钮，打开本地 `.csv` 文件选择框；
- 读取 CSV 文件内容，解析 `Address` 与 `Value` 字段；
- 将点位数值批量更新写入当前 Session 的寄存器内存块，并即时刷新数据网格显示。

---

## 3. 通信超时与重试参数控制 (Timeout & Retries)

### A. 设置弹窗新增参数 (`SettingsModal.vue`)
在连接参数设置弹窗中增加通信链路防护参数：
- **超时时间 (`timeoutMs`)**：数值输入框，默认 `1000` ms（范围 `100` ~ `10000` ms）。
- **重试次数 (`retries`)**：数值输入框，默认 `3` 次（范围 `0` ~ `10` 次）。

### B. 状态栏超时提示 (`StatusBar.vue`)
- 当通信握手超时或连续重试失败达到设定上限时，底部左下角状态栏即时呈报简短错误文字（如：`🔴 [Slave 2] 通信超时 (1000ms)`）；
- 自动暂停该连接周期的无限死等。

---

## 4. 显式排除的特性 (Non-Requirements)

- 本阶段不引入 `.json` / `.modlab` 私有文件格式，统一使用通用的标准 `.csv` 文件进行导入导出。
- 不引入复杂图表曲线绘制，专注于点位数据精准读写与显示。
