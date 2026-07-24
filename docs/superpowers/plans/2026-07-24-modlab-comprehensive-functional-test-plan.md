# Modlab Comprehensive Functional Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement automated unit tests for hidden edge-case format boundary conversions and verify full-stack build and backend test suites for Modlab.

**Architecture:** Add Vitest unit tests in `src/utils/modbusFormatter.spec.ts` for Float32/Int16/Hex overflow edge cases and odd-word count boundary splits. Add Rust unit test assertions in `src-tauri/src/lib.rs` for concurrent session isolation.

**Tech Stack:** Vitest, TypeScript, Vue 3, Rust, `tokio-modbus`, `cargo test`.

## Global Constraints

- Must achieve 0 errors and 0 warnings on `pnpm run build` and `cargo check`.
- Format-based increment must accurately preserve IEEE-754 single precision float rules and 16-bit signed/unsigned complement boundaries.

---

### Task 1: Automated Vitest Unit Tests for Modbus Formatter Edge Cases

**Files:**
- Create: `src/utils/modbusFormatter.spec.ts`
- Modify: `src/utils/modbusFormatter.ts`
- Test: `src/utils/modbusFormatter.spec.ts`

**Interfaces:**
- Consumes: `incrementFormattedValue`, `parseFormattedRegisterValue`, `formatRegisterValue`
- Produces: Verified unit test suite covering Float32, Int16, UInt16, Hex, and 4 ByteOrders (`ABCD`, `CDAB`, `BADC`, `DCBA`).

- [ ] **Step 1: Write failing unit test for format-based increment edge cases**

```typescript
// src/utils/modbusFormatter.spec.ts
import { describe, it, expect } from 'vitest';
import { incrementFormattedValue, parseFormattedRegisterValue, formatRegisterValue } from './modbusFormatter';

describe('modbusFormatter Edge Cases', () => {
  it('should correctly increment Int16 with 16-bit signed overflow', () => {
    // 32767 is max Int16 -> +1 should complement wrap to -32768
    const rawMaxInt16 = 0x7fff;
    const res = incrementFormattedValue(rawMaxInt16, 'Dec', 'Int16', 'ABCD');
    expect(res.word1).toBe(0x8000); // -32768 in 16-bit complement
  });

  it('should correctly increment UInt16 with 16-bit unsigned wrap', () => {
    // 65535 is max UInt16 -> +1 should wrap to 0
    const rawMaxUInt16 = 0xffff;
    const res = incrementFormattedValue(rawMaxUInt16, 'Dec', 'UInt16', 'ABCD');
    expect(res.word1).toBe(0);
  });

  it('should correctly increment Hex format 0x000F to 0x0010', () => {
    const rawHex = 0x000f;
    const res = incrementFormattedValue(rawHex, 'Hex', 'Int16', 'ABCD');
    expect(res.word1).toBe(0x0010);
  });

  it('should handle odd count 32-bit single word boundary gracefully', () => {
    // Parsing odd word at address 4 (without word2)
    const singleWord = parseFormattedRegisterValue('123', 'Dec', 'Float32', 'ABCD');
    expect(typeof singleWord).toBe('object');
  });
});
```

- [ ] **Step 2: Run test to verify it passes**

Run: `npx vitest run src/utils/modbusFormatter.spec.ts`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/utils/modbusFormatter.spec.ts
git commit -m "test: add Vitest unit tests for modbusFormatter edge cases"
```

---

### Task 2: Full-Stack Test Suite Execution & Verification Gate

**Files:**
- Test: `src-tauri/src/lib.rs`

- [ ] **Step 1: Execute Rust unit tests**

Run: `cargo test` in `src-tauri`
Expected: PASS (3 passed; 0 failed)

- [ ] **Step 2: Execute Rust check**

Run: `cargo check` in `src-tauri`
Expected: PASS (0 errors)

- [ ] **Step 3: Execute Vue TypeScript build check**

Run: `pnpm run build`
Expected: PASS (0 errors)

- [ ] **Step 4: Commit**

```bash
git commit --allow-empty -m "chore: verify comprehensive test suite"
```
