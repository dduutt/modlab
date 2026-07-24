# Task 1 Brief: 32-bit Float32 / Int32 & Endianness Engine (`src/utils/modbusFormatter.ts`)

## Environment & Constraints
- Package Manager: `pnpm`
- Framework: `Vue 3` (`<script setup lang="ts">`)
- Target: `src/utils/modbusFormatter.ts`

## Instructions
Update `src/utils/modbusFormatter.ts` to support 32-bit Float32 / Int32 / UInt32 formatting & parsing with 4-byte endianness swapping (`ABCD`, `CDAB`, `BADC`, `DCBA`).

Code for `src/utils/modbusFormatter.ts`:
```typescript
// Swaps 4 bytes based on Modbus byte order
function swapBytes32(bytes: Uint8Array, byteOrder: string = 'ABCD'): Uint8Array {
  const [a, b, c, d] = bytes;
  switch (byteOrder) {
    case 'CDAB': return new Uint8Array([c, d, a, b]); // Little-Endian Byte Swap
    case 'BADC': return new Uint8Array([b, a, d, c]); // Word Swap
    case 'DCBA': return new Uint8Array([d, c, b, a]); // Big-Endian Byte Swap
    case 'ABCD':
    default: return new Uint8Array([a, b, c, d]);     // Big-Endian
  }
}

function unswapBytes32(bytes: Uint8Array, byteOrder: string = 'ABCD'): Uint8Array {
  const [a, b, c, d] = bytes;
  switch (byteOrder) {
    case 'CDAB': return new Uint8Array([c, d, a, b]);
    case 'BADC': return new Uint8Array([b, a, d, c]);
    case 'DCBA': return new Uint8Array([d, c, b, a]);
    case 'ABCD':
    default: return new Uint8Array([a, b, c, d]);
  }
}

export function formatRegisterValue(
  raw: number,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  secondRaw?: number
): string {
  const low = raw ?? 0;
  
  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const high = secondRaw ?? 0;
    // Combine 2 16-bit words into 4 bytes
    const rawBytes = new Uint8Array([
      (low >> 8) & 0xff,
      low & 0xff,
      (high >> 8) & 0xff,
      high & 0xff,
    ]);

    const swapped = swapBytes32(rawBytes, byteOrder);
    const view = new DataView(swapped.buffer);

    if (dataType === 'Float32') {
      const val = view.getFloat32(0, false);
      return Number.isNaN(val) ? 'NaN' : val.toFixed(2);
    }
    if (dataType === 'Int32') {
      return view.getInt32(0, false).toString();
    }
    if (dataType === 'UInt32') {
      return view.getUint32(0, false).toString();
    }
  }

  if (format === 'Hex') {
    return '0x' + (low & 0xffff).toString(16).toUpperCase().padStart(4, '0');
  }
  return low.toString();
}

export function parseFormattedRegisterValue(
  input: string,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD'
): { word1: number; word2: number } | number {
  const trimmed = input.trim();

  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);

    if (dataType === 'Float32') {
      const floatVal = parseFloat(trimmed) || 0;
      view.setFloat32(0, floatVal, false);
    } else if (dataType === 'Int32') {
      const intVal = parseInt(trimmed, 10) || 0;
      view.setInt32(0, intVal, false);
    } else {
      const uintVal = parseInt(trimmed, 10) || 0;
      view.setUint32(0, uintVal, false);
    }

    const bytes = new Uint8Array(buffer);
    const unswapped = unswapBytes32(bytes, byteOrder);
    const word1 = (unswapped[0] << 8) | unswapped[1];
    const word2 = (unswapped[2] << 8) | unswapped[3];

    return { word1, word2 };
  }

  if (format === 'Hex' || trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return parseInt(trimmed.replace(/^0x/i, ''), 16) || 0;
  }
  return parseInt(trimmed, 10) || 0;
}
```

Write report to `file:///F:/pro/modlab/docs/superpowers/plans/task-ind-1-report.md`.
