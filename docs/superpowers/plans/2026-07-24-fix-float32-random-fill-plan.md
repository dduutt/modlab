# Fix Float32 Random/Increment Fill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `Float32`, `Int32`, `UInt32` random and increment fill generation in `modbusFormatter.ts` and `App.vue`.

**Architecture:** Add `generateRandomRegisters` and `generateIncrementRegisters` to `modbusFormatter.ts` and wire to `App.vue`.

**Tech Stack:** Vue 3, TypeScript, pnpm.

## Global Constraints
- Use `pnpm` for build verification.
- Preserve all existing functions.

---

### Task 1: Add DataType-Aware Random/Increment Generators in `modbusFormatter.ts`

**Files:**
- Modify: `src/utils/modbusFormatter.ts`

- [ ] **Step 1: Update formatRegisterValue and add generators**

In `src/utils/modbusFormatter.ts`:
```ts
// Update formatRegisterValue Float32 branch:
    if (dataType === 'Float32') {
      const val = view.getFloat32(0, false);
      return Number.isNaN(val) ? 'NaN' : parseFloat(val.toFixed(4)).toString();
    }

// Add export functions generateRandomRegisters & generateIncrementRegisters
```

---

### Task 2: Update `App.vue` to use Type-Aware Generators

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Wire generateRandomRegisters & generateIncrementRegisters in handleFillRandom and handleFillIncrement**

- [ ] **Step 2: Run verification with pnpm run build**
