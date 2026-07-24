# ConfigRow Stacked Label Layout Design Spec

## Objective
Place parameter labels directly above their respective input boxes/select dropdowns (vertical column flex layout) in `ConfigRow.vue` to save horizontal width and produce a clean, no-scrollbar single-row control bar.

## Design Details (`src/components/ConfigRow.vue`)
- Outer container: `flex items-center justify-between gap-1 px-2 py-1 bg-white border-b border-gray-100 text-xs select-none w-full min-h-[38px]`.
- Per-item container: `flex flex-col items-center gap-0.5 shrink-0`.
- Label styling: `text-[9px] font-bold text-gray-400 uppercase tracking-wider leading-none`.
- Input & Select styling: `h-5.5 px-1 border border-gray-200 rounded text-center text-xs font-medium bg-white focus:border-blue-500`.
- Control dimensions:
  - ID: input `w-11`
  - Func: select `h-5.5 px-1` (options: `0x01 Coils`, `0x02 Inputs`, `0x03 Holding`, `0x04 InputRegs`)
  - Start: input `w-14`
  - Count: input `w-14`
  - Type: select `h-5.5 px-1`
  - Fmt: select `h-5.5 px-1`
  - Order: select `h-5.5 px-1`
  - Intv: input `w-16`
  - Raw: switch `w-7 h-3.5 p-0.5`

## Verification
- Verify `ConfigRow.vue` template renders labels above controls cleanly without scrollbar.
- Run `pnpm run build` to verify type checking and bundle output.
