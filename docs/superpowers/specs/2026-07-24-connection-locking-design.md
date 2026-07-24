# Connection Parameter Locking Design Spec

## Objective
Implement smart parameter locking when connection is active (`connected === true`):
- `ConfigRow.vue`:
  - `Unit ID`: Locked if `role === 'Slave'` and `connected === true`. Unlocked if `role === 'Master'`.
  - `Function`, `Start`, `Count`: Locked when `connected === true`.
  - `Data Type`, `Format`, `Byte Order`, `Interval`, `Raw`: Always editable.
- `Toolbar.vue`:
  - `Settings` button: Disabled when `connected === true` with tooltip `"Disconnect to change connection settings"`.

## Verification
- Verify that `ConfigRow.vue` locks Function, Start, Count when connected.
- Verify that `ConfigRow.vue` locks Unit ID only when connected in Slave mode, but allows editing in Master mode.
- Verify `Settings` button is disabled when connected.
- Run `pnpm run build` to verify type checking and bundle output.
