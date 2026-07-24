# Task 2 Report: DataGrid 2-Register Cell Display & Editing (`DataGrid.vue`)

## Execution Summary
- **Target File**: `src/components/DataGrid.vue` & `src/App.vue`
- **Status**: Completed & Verified (`pnpm build` passed without errors)

## Changes Made
1. **`src/components/DataGrid.vue`**:
   - Added `byteOrder?: string` to component `props`.
   - Defined `is32Bit` computed property to detect `Float32`, `Int32`, and `UInt32` data types.
   - Added `update-cell-pair` emit to component `emits` signature: `(e: 'update-cell-pair', address1: number, value1: number, address2: number, value2: number): void`.
   - Updated `startEdit(addr)` to fetch `raw1` (`values[addr]`) and `raw2` (`values[addr + 1]`), passing both into `formatRegisterValue` with `props.dataType` and `props.byteOrder`.
   - Updated `saveEdit(addr)` to parse formatted string with `parseFormattedRegisterValue`. If an object `{ word1, word2 }` is returned (for 32-bit types), it emits `update-cell-pair`. Otherwise, it emits `update-cell`.
   - Updated `copyValue()` to copy the 32-bit formatted text when in 32-bit mode using both register words.
   - Updated `resetToZero()` to emit `update-cell-pair` with `(addr, 0, addr+1, 0)` if `is32Bit` is true.
   - Updated table cell template to pass `props.dataType`, `props.byteOrder`, and the high/adjacent word (`props.values[addr + 1]`) to `formatRegisterValue`.

2. **`src/App.vue`**:
   - Passed `:byteOrder="activeTab.config.byteOrder"` to `<DataGrid />`.
   - Added `@update-cell-pair="handleUpdateCellPair"` event listener to `<DataGrid />`.
   - Implemented `handleUpdateCellPair(address1, value1, address2, value2)` in `src/App.vue` to update both 16-bit registers locally and persist/sync via `ModbusService.writeRegister`.

## Verification Results
- Ran `pnpm build` (`vue-tsc --noEmit && vite build`), which compiled successfully with 0 errors.
