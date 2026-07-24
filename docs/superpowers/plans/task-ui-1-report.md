# Task 1 Implementation Report: New Device Modal (`NewDeviceModal.vue`)

## Overview
Successfully implemented `NewDeviceModal.vue` and integrated it into `src/App.vue`. Users can now click the `+` button in the tab bar to open a modal dialog allowing them to specify a custom device name and select the device role (`Slave` or `Master`) before creating the new session tab.

## Changes Made
1. **Created `src/components/NewDeviceModal.vue`**:
   - Built styled modal dialog using TailwindCSS with backdrop blur, fade/zoom animations, and Lucide icons (`Plus`, `X`, `Server`, `Cpu`).
   - Defined props (`show: boolean`) and emits (`close`, `create(deviceName: string, role: 'Slave' | 'Master')`).
   - Provided interactive role selector buttons switching state between `Slave` and `Master`.
   - Handled default device fallback naming (`Slave 2` or `Master 1` if input is blank).

2. **Updated `src/App.vue`**:
   - Added `showNewDeviceModal` reactive state.
   - Connected `TabBar`'s `@add-tab` event to open `showNewDeviceModal = true`.
   - Implemented `handleCreateDevice(name: string, role: 'Slave' | 'Master')` to instantiate a new session tab configured with the specified role, title, and initial connection/Modbus parameters.
   - Included `<NewDeviceModal>` at the root view level.

## Verification
- Executed `pnpm run build` (`vue-tsc --noEmit && vite build`).
- Confirmed zero TypeScript or build errors.
