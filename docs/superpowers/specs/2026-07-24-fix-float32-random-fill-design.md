# Fix Float32 Random/Increment Fill Design Spec

## Root Cause Analysis
Previously, `fillRandom` and `fillIncrement` generated arbitrary small 16-bit integers (e.g. `val < 1000`), which resulted in IEEE-754 Float32 high-word exponent bits being 0 (`0x00XX`). In IEEE-754, an exponent of 0 indicates a denormalized float (~10^-39), which rounds to `0.00` when formatted. Thus, clicking `Random` or `Increment` while `dataType` was `Float32` appeared to be broken and produced `0.00` everywhere.

## Solution Details
1. In `src/utils/modbusFormatter.ts`:
   - Update `formatRegisterValue` to output Float32 values with up to 4 decimal places (`parseFloat(val.toFixed(4)).toString()`).
   - Implement `generateRandomRegisters(start, count, dataType, byteOrder, functionCode)`.
   - Implement `generateIncrementRegisters(start, count, dataType, byteOrder, functionCode)`.
2. In `src/App.vue`:
   - Update `handleFillRandom()` and `handleFillIncrement()` to invoke type-aware `generateRandomRegisters` and `generateIncrementRegisters`, updating session memory and active values.

## Verification
- Verify that clicking `Random` when `dataType` is `Float32` populates realistic, non-zero floating point numbers (e.g. `823.4381`, `-3626.063`).
- Verify that `Increment` when `dataType` is `Float32` populates incrementing floats.
- Run `pnpm run build` to verify clean build without type errors.
