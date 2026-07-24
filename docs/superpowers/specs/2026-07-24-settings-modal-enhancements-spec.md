# Settings Modal Enhancements Spec

## Requirements
1. Role Locking: Once a device tab is created, its `Slave` / `Master` role in `SettingsModal.vue` cannot be modified (buttons disabled with fixed note).
2. RTU Serial Port Dropdown: When `RTU` protocol is selected in `SettingsModal.vue`, change Serial Port from plain text input to a `<select>` dropdown supporting `COM1` ~ `COM16`, `/dev/ttyUSB0` ~ `/dev/ttyUSB3`, and preserving custom values.

## Verification
- Verify `SettingsModal.vue` disables role selection buttons.
- Verify `SettingsModal.vue` displays a dropdown for Serial Port selection in RTU mode.
- Run `pnpm run build` to verify clean build without type errors.
