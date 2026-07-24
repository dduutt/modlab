# Match 示例.png Design Spec

## Objective
Styling and layout match with `示例.png`:
- Full label names ("Unit ID", "Function", "Start", "Count", "Data Type", "Format", "Byte Order", "Interval", "Raw").
- Rounded pill style for inputs and dropdown selects (`rounded-full` or `rounded-xl`).
- Clean stacked layout with labels above controls, balanced spacing without overflow or scrollbars.

## Design Details

### 1. `src/components/ConfigRow.vue`
- Outer container: `flex items-center justify-between gap-2.5 px-4 py-2 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[50px]`.
- Per item container: `flex flex-col items-center gap-1 shrink-0`.
- Label styling: `text-xs font-semibold text-gray-600 select-none`.
- Control styling: `h-8 px-3 border border-gray-200 rounded-full outline-none focus:border-blue-500 text-center font-medium bg-white text-xs shadow-2xs`.
- Label text & widths:
  - Unit ID: Label `Unit ID`, input `w-16`
  - Function: Label `Function`, select `px-3` with options (`Holding (0x03)`, `Read Coils (0x01)`, `Read Discrete Inputs (0x02)`, `Read Input Registers (0x04)`)
  - Start Address: Label `Start`, input `w-16`
  - Count: Label `Count`, input `w-16`
  - Data Type: Label `Data Type`, select `px-3`
  - Format: Label `Format`, select `px-3`
  - Byte Order: Label `Byte Order`, select `px-3`
  - Interval: Label `Interval`, input `w-20`
  - Raw: Label `Raw`, toggle switch `w-9 h-5 rounded-full p-0.5` with thumb `w-4 h-4 rounded-full translate-x-4` / `translate-x-0`

### 2. `src/components/Toolbar.vue`
- Match `示例.png` clean pill styling for Connect, Settings, Traffic Log, Random, Increment, Export, Import.
- Container: `flex items-center justify-between px-6 py-2.5 bg-white border-b border-gray-200 select-none text-xs`.
- Rounded pill buttons: `rounded-full px-4 py-1.5 text-xs font-medium border border-gray-200`.

## Verification
- Verify build with `pnpm run build`.
