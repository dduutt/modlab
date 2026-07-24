# Task 3 Report: CSV Import & Export Utilities (`csvHandler.ts` & `Toolbar.vue` / `App.vue`)

## Executive Summary
Task 3 of the Industrial Modbus Enhancements plan has been successfully completed. 
The CSV import and export utility function module `src/utils/csvHandler.ts` was created, and UI controls for CSV export and import were added to `src/components/Toolbar.vue` and connected in `src/App.vue`.

## Changes Summary

### 1. Created `src/utils/csvHandler.ts`
- **`exportToCSV(sessionName, values)`**:
  - Converts active session's register map (`Record<number, number>`) into standard CSV format with `Address,Value` headers.
  - Sorts address keys numerically before generating lines.
  - Creates a Blob and triggers an automated browser file download named `${sessionName.replace(/\s+/g, '_')}_registers.csv`.
- **`parseCSVFile(file)`**:
  - Reads a user-uploaded CSV file as text asynchronously using `FileReader`.
  - Skips empty lines and header rows (starting with `Address`).
  - Parses comma-separated `address,value` pairs, validating numeric entries via `parseInt` and `!isNaN`.
  - Returns a `Promise<Record<number, number>>`.

### 2. Updated `src/components/Toolbar.vue`
- Imported `Download` and `Upload` icons from `@lucide/vue`.
- Defined `export-csv` and `import-csv` emits.
- Added Export and Import action buttons to the toolbar action group.
- Added a hidden `<input type="file" accept=".csv" ref="fileInput">` element that opens when clicking the Import button.
- Disabled the Import button when `functionCode` is read-only (`0x02` or `0x04`).

### 3. Updated `src/App.vue`
- Imported `exportToCSV` and `parseCSVFile` from `src/utils/csvHandler.ts`.
- Implemented `handleExportCSV` to trigger CSV generation for the active tab's values.
- Implemented `handleImportCSV(file)` to parse the selected file, merge imported registers into `activeTab.value.values`, and update the backend via `ModbusService.writeRegister`.
- Passed `@export-csv` and `@import-csv` handlers to `<Toolbar />`.

## Files Created / Modified
- `src/utils/csvHandler.ts` (New file)
- `src/components/Toolbar.vue` (Modified)
- `src/App.vue` (Modified)
- `docs/superpowers/plans/task-ind-3-report.md` (New report file)

## Verification
- Verified code structure against TypeScript and Vue 3 standards.
- CSV parsing and formatting logic tested against edge cases (empty text, header skipping, non-numeric values).
