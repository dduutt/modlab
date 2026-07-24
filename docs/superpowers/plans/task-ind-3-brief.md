# Task 3 Brief: CSV Import & Export Utilities (`csvHandler.ts` & `Toolbar.vue` / `App.vue`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Icons: `@lucide/vue`

## Instructions
1. Create `src/utils/csvHandler.ts` providing `exportToCSV` and `parseCSVFile`.
2. Update `src/components/Toolbar.vue` to add `Import` button alongside `Export`, triggering hidden `<input type="file" accept=".csv">`.
3. Update `src/App.vue` to handle `@export-csv` and `@import-csv`.

Code for `src/utils/csvHandler.ts`:
```typescript
export function exportToCSV(sessionName: string, values: Record<number, number>) {
  let csvContent = 'Address,Value\n';
  const addresses = Object.keys(values).map(Number).sort((a, b) => a - b);
  for (const addr of addresses) {
    csvContent += `${addr},${values[addr]}\n`;
  }

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${sessionName.replace(/\s+/g, '_')}_registers.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function parseCSVFile(file: File): Promise<Record<number, number>> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const text = e.target?.result as string;
      if (!text) {
        return resolve({});
      }
      const lines = text.split(/\r?\n/);
      const result: Record<number, number> = {};
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        if (!line || line.toLowerCase().startsWith('address')) continue; // Skip header
        const [addrStr, valStr] = line.split(',');
        const addr = parseInt(addrStr?.trim(), 10);
        const val = parseInt(valStr?.trim(), 10);
        if (!isNaN(addr) && !isNaN(val)) {
          result[addr] = val;
        }
      }
      resolve(result);
    };
    reader.onerror = (err) => reject(err);
    reader.readAsText(file);
  });
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-ind-3-report.md`.
