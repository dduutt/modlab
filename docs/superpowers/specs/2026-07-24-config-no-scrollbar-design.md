# ConfigRow No-Scrollbar Design Spec

## Objective
Completely remove horizontal scrollbar from `ConfigRow.vue` by trimming label text, option text, padding, and control widths so that all 9 configuration parameters fit across a single row without horizontal overflow on small to medium windows.

## Design Details (`src/components/ConfigRow.vue`)
- Outer container: `flex items-center justify-between gap-1.5 px-2 py-1 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[34px]`.
- Remove: `overflow-x-auto` and `whitespace-nowrap`.
- Labels & Fields:
  - Unit ID: Label `ID`, input `w-11 h-6 text-center text-xs px-1 rounded border-gray-200`.
  - Function: Label `Func`, select `h-6 text-xs px-1 rounded border-gray-200`.
    - Options: `0x01 Coils`, `0x02 Inputs`, `0x03 Holding`, `0x04 InputRegs`.
  - Start Address: Label `Start`, input `w-14 h-6 text-center text-xs px-1 rounded border-gray-200`.
  - Count: Label `Count`, input `w-14 h-6 text-center text-xs px-1 rounded border-gray-200`.
  - Data Type: Label `Type`, select `h-6 text-xs px-1 rounded border-gray-200`.
  - Format: Label `Fmt`, select `h-6 text-xs px-1 rounded border-gray-200`.
  - Byte Order: Label `Order`, select `h-6 text-xs px-1 rounded border-gray-200`.
  - Interval: Label `Intv`, input `w-16 h-6 text-center text-xs px-1 rounded border-gray-200`.
  - Raw: Label `Raw`, toggle switch `w-7 h-3.5 p-0.5 rounded-full` with thumb `w-2.5 h-2.5 rounded-full translate-x-3`.

## Verification
- Verify `ConfigRow.vue` renders across a single line with no scrollbar.
- Run `pnpm run build` to verify type checking and bundle output.
