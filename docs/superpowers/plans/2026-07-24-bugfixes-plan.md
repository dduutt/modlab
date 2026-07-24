# System Bugfixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 4 system bugfixes and UI consistency improvements.

**Architecture:** Modify `modbusFormatter.ts`, `TabBar.vue`, `DataGrid.vue`, and `ConfigRow.vue`.

**Tech Stack:** Vue 3, TypeScript, pnpm.

## Global Constraints
- Use `pnpm` for build verification.
- Preserve all existing functionality.

---

### Task 1: Fix `Int16` Signed Formatting in `modbusFormatter.ts`

**Files:**
- Modify: `src/utils/modbusFormatter.ts`

- [ ] **Step 1: Update formatRegisterValue Int16 handling**

---

### Task 2: Fix `TabBar.vue` Connection Indicator

**Files:**
- Modify: `src/components/TabBar.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: Pass connected field in TabBar tabs and update green dot rendering**

---

### Task 3: Fix `DataGrid.vue` Escape Key Cancel Edit

**Files:**
- Modify: `src/components/DataGrid.vue`

- [ ] **Step 1: Add cancelEdit handler on @keyup.esc**

---

### Task 4: Disable Irrelevant Controls for Coils in `ConfigRow.vue`

**Files:**
- Modify: `src/components/ConfigRow.vue`

- [ ] **Step 1: Add disabled condition for dataType and byteOrder when functionCode is 0x01 or 0x02**

- [ ] **Step 2: Run verification with pnpm run build**
