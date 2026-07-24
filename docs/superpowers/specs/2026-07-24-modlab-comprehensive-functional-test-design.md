# Modlab 隐蔽死角与核心功能测试设计规范 (Comprehensive Functional Test Design)

## 1. 概述 (Overview)

本规范专门针对 Modlab 工业 Modbus 工具软件在实际工业现场可能遇到的**隐蔽数据死角、多字节序反向编解码、高频轮询内存溢出、定时器句柄泄漏及异常断网边界**，设计一套精准、高覆盖率的硬核功能测试方案。

---

## 2. 隐蔽边界与死角测试矩阵 (Hidden Edge-Case Test Matrix)

### 2.1 隐蔽死角一：32 位双字跨边界奇数截断 (Odd Count Boundary Split)
- **触发条件**：设置 `Count: 5`（奇数），数据类型切换为 `Float32` 或 `Int32`。
- **内存分布**：地址 `0-1`（双字对 1）、`2-3`（双字对 2）、**地址 `4`（被孤立的单 Word 寄存器）**。
- **校验点**：
  1. 双击编辑地址 `4` 时，系统不崩溃，在单寄存器与双寄存器解析模式间平滑防错。
  2. 写回数据时不发生底层数组 `IndexOutOfBounds` 错误。

### 2.2 隐蔽死角二：4 种 ByteOrder 字节序下 32 位高低字反向写回 (Endianness Reverse Encoding)
- **触发条件**：切换 `ByteOrder` 为 `CDAB` 或 `DCBA`。
- **测试步骤**：
  1. 在 `CDAB` 模式下双击写入 `Float32` 数值 `1234.56`。
  2. 将 `ByteOrder` 切换为 `ABCD` 观察底层原始 16 位 Word 1 与 Word 2。
- **校验点**：写回底层 Rust `SessionMemory` 时，高低字与高低字节严格按逆向 `CDAB` 重新拼接写回，避免读展示正确但写数据错乱。

### 2.3 隐蔽死角三：Float32 浮点累积误差与边界溢出回卷 (Float32 Precision Accumulation & Boundary Wrap)
- **触发条件**：开启从站 `Auto Increment` 自增。
- **测试场景**：
  - **Float32 场景**：原值设为 `16777216.0`（$2^{24}$ 单精度浮点有效位数上限），验证自增是否会因 IEEE-754 精度丢失而陷入死循环。
  - **Int16 场景**：原值设为 `32767`，验证自增 `+1` 后是否正确补码溢出为 `-32768`（不变成 `32768`）。
  - **UInt16 场景**：原值设为 `65535`，验证自增 `+1` 后是否正确回卷为 `0`。

### 2.4 隐蔽死角四：高频 50ms 轮询下的 DOM 节点与日志内存泄露 (Memory & Log Overflow)
- **触发条件**：`Interval` 设为 `50ms`，开启 `Poll` 连续运行 3 分钟（产生约 3,600 条报文）。
- **校验点**：
  1. 日志数组元素上限严格限制在 `max 100` 条以内，超过自动弹出旧日志。
  2. 打开日志抽屉时，界面 FPS 稳定在 60 帧，DOM 节点数维持稳定，无内存泄漏。

### 2.5 隐蔽死角五：快速频繁切换 Tab 与断开连接时的僵尸句柄泄露 (Zombie Timers)
- **触发条件**：
  1. 在 `Tab 1` 开启 50ms 高频 Polling。
  2. 不手动停止 Polling，直接关闭 `Tab 1` 或断开连接。
- **校验点**：
  1. `tabTimers` Map 映射表中对应 Tab ID 的 `setInterval` 句柄被物理 `clearInterval`。
  2. 任务管理器中 CPU 占用率归零，无孤立僵尸线程在后台静默轮询。

---

## 3. 核心功能开关与联动测试 (Core Toggle & Interaction Matrix)

| 编号 | 模块 | 操作 | 预期隐蔽行为 |
| :--- | :--- | :--- | :--- |
| **TG-01** | **主站 Connect** | 主站处于 `Poll` 轮询状态时，点击 `Disconnect` | 必须同步将 `isPolling` 设为 `false`，并同步物理清理轮询定时器。 |
| **TG-02** | **从站 Listen** | 从站处于 `Auto Increment` 状态时，点击 `Stop` | 必须同步将 `autoIncrement` 设为 `false`，并冻结当前寄存器数值。 |
| **TG-03** | **IPC 错误呈递** | 主站输入非法/不可达 IP（如 `0.0.0.0` 或断网 IP）点击 `Connect` | Rust 后端通过 `Result::Err` 将 `TCP Connect Error` 抛给前端，状态栏变红呈递报错。 |

---

## 4. 自动化测试套件设计 (Automated Test Suite)

1. **前端单元测试 (`src/utils/modbusFormatter.spec.ts`)**：
   - 验证 `incrementFormattedValue` 在 Float32、Int16、UInt16、Hex、Coil 下的溢出回卷逻辑。
   - 验证 4 种 ByteOrder 字节序的双向转码取值。
2. **Rust 后端并发与隔离测试 (`src-tauri/src/lib.rs` `tests`)**：
   - 验证多 Tab `AppState` 下 `SessionMemory` 的独立加锁与线程安全。
3. **构建流程校验**：
   - 执行 `cargo test` $\to$ `cargo check` $\to$ `pnpm run build`（0 Warnings, 0 Errors）。
