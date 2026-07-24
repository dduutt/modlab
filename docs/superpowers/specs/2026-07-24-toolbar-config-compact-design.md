# Compact Layout Design Spec for Toolbar and ConfigRow

## Objective
Optimize `Toolbar.vue` and `ConfigRow.vue` components to fit compact window heights and widths, preventing layout overflow, excessive vertical padding, and unwanted row wrapping that squeezes the main `DataGrid.vue`.

## Target Components

### 1. `src/components/Toolbar.vue`
- Outer container padding reduced from `px-4 py-3` to `px-3 py-1.5`.
- Overall height target: ~38px.
- Left group (Role, Protocol, IP:Port, Connect/Disconnect button, Settings button, Traffic Log button):
  - Compact text font size `text-xs`.
  - Buttons styled with `px-2.5 py-1 text-xs rounded-md`.
  - Lucide icons sized to `w-3.5 h-3.5`.
- Right group (Random, Increment, Export CSV, Import CSV):
  - Buttons styled with `px-2.5 py-1 text-xs rounded-md`.
  - Flexible gap (`gap-2` instead of `gap-3`).

### 2. `src/components/ConfigRow.vue`
- Outer container padding reduced from `px-6 py-4` to `px-3 py-1.5`.
- Overall height target: ~36px.
- Single-row layout with horizontal scroll on small viewports: `flex items-center gap-3 overflow-x-auto whitespace-nowrap min-h-[36px]`.
- Field labels: `text-[10px] uppercase font-medium text-gray-500`.
- Field inputs/selects: `h-7 px-2 text-xs rounded-md border-gray-200 focus:border-blue-500`.
- Field width limits:
  - Unit ID: `w-14`
  - Function: auto
  - Start: `w-16`
  - Count: `w-16`
  - Data Type: auto
  - Format: auto
  - Byte Order: auto
  - Interval: `w-20`
  - Raw switch: `w-8 h-4` toggle button with `w-3.5 h-3.5` thumb.

## Verification
- Verify that `Toolbar.vue` and `ConfigRow.vue` render compactly without wrapping or vertical overflow.
- Verify `DataGrid.vue` receives maximum available vertical height.
- Run `npm run build` or `vue-tsc` to ensure no TypeScript/Vue syntax errors.
