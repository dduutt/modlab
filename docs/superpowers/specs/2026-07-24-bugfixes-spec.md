# System Bugfixes Spec

## Objectives
Fix 4 identified system bugs and UI inconsistencies across the project:
1. `Int16` signed negative integer formatting in `modbusFormatter.ts`.
2. `TabBar.vue` connection status indicator (green dot when `connected === true`).
3. `DataGrid.vue` cell edit cancellation on `Escape` key (`@keyup.esc`).
4. `ConfigRow.vue` disabling irrelevant `dataType` and `byteOrder` when `functionCode` is `0x01` or `0x02`.

## Detailed Fixes

### 1. `src/utils/modbusFormatter.ts`
- Update `formatRegisterValue` for `Int16`:
  ```ts
  if (dataType === 'Int16' && format !== 'Hex') {
    const signed16 = low > 32767 ? low - 65536 : low;
    return signed16.toString();
  }
  if (dataType === 'UInt16' && format !== 'Hex') {
    return (low & 0xffff).toString();
  }
  ```

### 2. `src/components/TabBar.vue` & `src/App.vue`
- Update `Tab` interface in `TabBar.vue` to include `connected?: boolean`.
- In `TabBar.vue` template:
  Render status dot as `bg-emerald-500` if `tab.connected` is true, otherwise `bg-blue-500` for active or `bg-gray-300` for inactive.

### 3. `src/components/DataGrid.vue`
- Add `@keyup.esc="cancelEdit"` to cell edit input element.
- Define `cancelEdit()` method (`editingAddress.value = null`).

### 4. `src/components/ConfigRow.vue`
- Disable `Data Type` and `Byte Order` selects when `functionCode === '0x01' || functionCode === '0x02'`.

## Verification
- Verify `pnpm run build` succeeds cleanly with zero errors.
