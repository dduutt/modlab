# Modlab System Architecture & Functional Requirements Design Spec

## Overview
Modlab is a professional, high-performance Modbus Master/Slave simulation and testing tool built with Vue 3, TypeScript, Tailwind CSS, and Tauri.
This document defines the complete re-aligned functional requirements, UI workflows, and state interactions for Master and Slave modes.

---

## 1. Master vs Slave Role Workflows & Toolbars

### 1.1 Slave (Server) Mode
In Slave mode, Modlab simulates a Modbus server listening for external master requests while providing simulation drivers to model real-time changing field data.

- **Toolbar Buttons**:
  - `Listen / Stop`: Starts or stops local Modbus server listener.
  - `Settings`: Configures local listening parameters (IP, Port, Serial Port, Baud Rate, Parity, Timeout).
  - `Auto Increment` (Toggle): When ON, every `Interval` ms, the software automatically increments the **parsed/formatted value** of each register/pair by `+1` (e.g. `823.4381` -> `824.4381`, `10` -> `11`, `0x000A` -> `0x000B`, `0` -> `1` -> `0`), re-encoding the result into session memory.
  - `Random` (Action Button): One-time trigger that populates realistic random values for all registers conforming to current `Data Type` and `Format`.
- **Data Editing**:
  - Double-clicking a cell directly updates local session memory.

### 1.2 Master (Client) Mode
In Master mode, Modlab acts as an active client polling remote Modbus servers or sending single read/write commands.

- **Toolbar Buttons**:
  - `Connect / Disconnect`: Establishes or terminates connection with target Slave device.
  - `Settings`: Configures target device connection parameters (IP, Port, Serial Port, Baud Rate, Parity, Timeout, Retries).
  - `Poll` (Toggle): When ON, periodically sends Modbus Read requests every `Interval` ms to update the grid.
  - `Read` (Action Button): Sends a single Modbus Read request (Function 0x01/0x02/0x03/0x04) to target device and updates the grid.
- **Data Editing**:
  - Double-clicking a cell sends a Modbus Write command (Function 0x06 for single register / 0x10 for multiple registers) to target device. Updates local state upon successful response.

---

## 2. Configuration & Parameter Rules (`ConfigRow.vue`)

- **`Unit ID`**:
  - Master mode: Editable while connected (to allow querying multiple slave IDs on the same bus).
  - Slave mode: Locked/disabled while connected.
- **`Function Code` / `Start Address` / `Count`**:
  - Locked/disabled while connected.
- **`Data Type` / `Format` / `Byte Order`**:
  - Disabled when `Function Code` is `0x01` (Coils) or `0x02` (Discrete Inputs).
  - Editable when `0x03` (Holding Registers) or `0x04` (Input Registers).
- **`Raw`**:
  - Toggle ON/OFF to display 16-bit hex subtext (`0xXXXX`) under formatted cell values.

---

## 3. Data Grid & Cell Display (`DataGrid.vue`)

- **10-Column Grid**: `Address` column (width `w-20`) + 10 data columns (`0` through `9`).
- **32-Bit Types (Float32, Int32, UInt32)**:
  - Spans 2 consecutive registers: Address $N$ and Address $N+1$.
  - Address $N$ displays the full parsed 32-bit value.
  - Address $N+1$ displays `-` for primary text.
  - Double-clicking or right-clicking Address $N+1$ automatically targets the 32-bit pair at Address $N$.
- **16-Bit Signed Integer (Int16)**:
  - Values $\ge 32768$ display as signed negative numbers (e.g. `0xFFFF` -> `-1`).
- **Keyboard Shortcuts**:
  - `Enter`: Saves cell edit.
  - `Escape`: Cancels cell edit without saving.

---

## 4. App Startup & Configuration Persistence (`App.vue`)

- **Persistence Storage**: `localStorage` key `'modlab_session_configs_v1'`.
- **Persisted Attributes**:
  - Tab list, Tab titles, active Tab ID.
  - `connection` settings (role, protocol, IP, port, serialPort, baudRate, parity, etc.).
  - `config` parameters (unitId, functionCode, startAddress, count, dataType, format, byteOrder, interval, raw).
- **Startup Reset Policy**:
  - `connected = false` for all tabs.
  - `values = {}` (empty register data) for all tabs.
  - `logs = []` (empty logs) for all tabs.

---

## 5. Summary of Component Responsibilities

1. **`Toolbar.vue`**: Render role-specific compact controls (Slave: `Listen`, `Settings`, `Auto Increment`, `Random`; Master: `Connect`, `Settings`, `Poll`, `Read`).
2. **`ConfigRow.vue`**: Parameter input fields with role/connection locking rules and coil-disabling rules.
3. **`DataGrid.vue`**: Table rendering with format-based `Auto Increment` helper, 32-bit pairing, signed Int16 formatting, and Esc cancel key.
4. **`SettingsModal.vue`**: Connection settings with read-only role badge in header and COM port dropdown select (`COM1`~`COM16`, `/dev/ttyUSB*`).
5. **`NewDeviceModal.vue`**: Device tab creation with auto-suggested device names (`Slave X`, `Master Y`).
