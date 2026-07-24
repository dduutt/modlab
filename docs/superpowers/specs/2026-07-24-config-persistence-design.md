# Configuration Persistence Design Spec

## Requirement
"启动只需要记住配置，不需要记住值和连接状态，启动后都是未连接状态"
- On app startup, persist and reload tab list, device names, connection settings, and Modbus configurations from `localStorage`.
- Do NOT persist register `values`, `logs`, or `connected` states.
- On startup, ensure all tabs default to `connected = false`, `values = {}`, and `logs = []`.

## Implementation Details in `src/App.vue`
1. Define `STORAGE_KEY = 'modlab_session_configs_v1'`.
2. Helper `loadSavedTabs()`:
   - Reads `localStorage.getItem(STORAGE_KEY)`.
   - Parses stored array of `{ id, title, connection, config }`.
   - Returns initialized `SessionTab` objects with `connected: false`, `values: {}`, `logs: []`, `statusMessage: '[Title] Ready.'`, `statusType: 'info'`.
3. Helper `saveTabs()`:
   - Saves array of `{ id, title, connection, config }` and `activeTabId` to `localStorage`.
4. Deep watcher on `tabs` and `activeTabId` to call `saveTabs()`.

## Verification
- Verify `pnpm run build` passes with zero TypeScript or Vite compilation errors.
