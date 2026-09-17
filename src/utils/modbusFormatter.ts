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

    if (format === 'Hex') {
      return '0x' + view.getUint32(0, false).toString(16).toUpperCase().padStart(8, '0');
    }

    if (dataType === 'Float32') {
      const val = view.getFloat32(0, false);
      return val.toString();
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
  if (dataType === 'Int16') {
    const signed16 = low > 32767 ? low - 65536 : low;
    return signed16.toString();
  }
  return (low & 0xffff).toString();
}

export function parseFormattedRegisterValue(
  input: string,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD'
): { word1: number; word2: number } | number {
  const trimmed = input.trim();
  const is32 = dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32';
  const isHex = format === 'Hex' || /^0x/i.test(trimmed);
  let value: number;
  if (isHex) {
    if (!/^(?:0x)?[0-9a-f]+$/i.test(trimmed)) throw new Error('Invalid hexadecimal value');
    value = Number.parseInt(trimmed.replace(/^0x/i, ''), 16);
    if (value > (is32 ? 0xffffffff : 0xffff)) throw new Error('Value out of range');
  } else {
    const pattern = dataType === 'Float32' ? /^[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?$/i : /^[+-]?\d+$/;
    if (!pattern.test(trimmed)) throw new Error('Invalid numeric value');
    value = Number(trimmed);
    if (!Number.isFinite(value)) throw new Error('Value out of range');
    if (dataType === 'Float32') {
      if (!Number.isFinite(Math.fround(value))) throw new Error('Value out of range');
    } else {
      const min = dataType === 'Int32' ? -2147483648 : dataType === 'Int16' ? -32768 : 0;
      const max = dataType === 'Int32' ? 2147483647 : dataType === 'UInt32' ? 4294967295 : dataType === 'Int16' ? 32767 : 65535;
      if (!Number.isInteger(value) || value < min || value > max) throw new Error('Value out of range');
    }
  }

  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);

    if (isHex) {
      view.setUint32(0, value, false);
    } else if (dataType === 'Float32') {
      view.setFloat32(0, value, false);
    } else if (dataType === 'Int32') {
      view.setInt32(0, value, false);
    } else {
      view.setUint32(0, value, false);
    }

    const bytes = new Uint8Array(buffer);
    const unswapped = unswapBytes32(bytes, byteOrder);
    const word1 = (unswapped[0] << 8) | unswapped[1];
    const word2 = (unswapped[2] << 8) | unswapped[3];

    return { word1, word2 };
  }

  return value & 0xffff;
}

export function generateRandomRegisters(
  start: number,
  count: number,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  functionCode?: string
): Record<number, number> {
  const result: Record<number, number> = {};
  const isCoil = functionCode === '0x01' || functionCode === '0x02';

  if (isCoil) {
    for (let addr = start; addr < start + count; addr++) {
      result[addr] = Math.random() > 0.5 ? 1 : 0;
    }
    return result;
  }

  if (dataType === 'Float32') {
    for (let addr = start; addr < start + count; addr += 2) {
      const rawFloat = (Math.random() * 10000 - 5000).toFixed(4);
      const parsed = parseFormattedRegisterValue(rawFloat, 'Dec', 'Float32', byteOrder);
      if (typeof parsed === 'object') {
        result[addr] = parsed.word1;
        if (addr + 1 < start + count) {
          result[addr + 1] = parsed.word2;
        }
      }
    }
    return result;
  }

  if (dataType === 'Int32' || dataType === 'UInt32') {
    for (let addr = start; addr < start + count; addr += 2) {
      const rawInt = dataType === 'Int32'
        ? Math.floor(Math.random() * 200000 - 100000)
        : Math.floor(Math.random() * 200000);
      const parsed = parseFormattedRegisterValue(rawInt.toString(), 'Dec', dataType, byteOrder);
      if (typeof parsed === 'object') {
        result[addr] = parsed.word1;
        if (addr + 1 < start + count) {
          result[addr + 1] = parsed.word2;
        }
      }
    }
    return result;
  }

  for (let addr = start; addr < start + count; addr++) {
    result[addr] = Math.floor(Math.random() * 1000);
  }

  return result;
}

export function generateIncrementRegisters(
  start: number,
  count: number,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  functionCode?: string
): Record<number, number> {
  const result: Record<number, number> = {};
  const isCoil = functionCode === '0x01' || functionCode === '0x02';

  if (isCoil) {
    let val = 1;
    for (let addr = start; addr < start + count; addr++) {
      result[addr] = val;
      val = val === 1 ? 0 : 1;
    }
    return result;
  }

  if (dataType === 'Float32') {
    let floatVal = 100.5;
    for (let addr = start; addr < start + count; addr += 2) {
      const parsed = parseFormattedRegisterValue(floatVal.toFixed(2), 'Dec', 'Float32', byteOrder);
      if (typeof parsed === 'object') {
        result[addr] = parsed.word1;
        if (addr + 1 < start + count) {
          result[addr + 1] = parsed.word2;
        }
      }
      floatVal += 100.5;
    }
    return result;
  }

  if (dataType === 'Int32' || dataType === 'UInt32') {
    let intVal = 1000;
    for (let addr = start; addr < start + count; addr += 2) {
      const parsed = parseFormattedRegisterValue(intVal.toString(), 'Dec', dataType, byteOrder);
      if (typeof parsed === 'object') {
        result[addr] = parsed.word1;
        if (addr + 1 < start + count) {
          result[addr + 1] = parsed.word2;
        }
      }
      intVal += 1000;
    }
    return result;
  }

  let stepVal = 1;
  for (let addr = start; addr < start + count; addr++) {
    result[addr] = stepVal++;
  }

  return result;
}

export function incrementFormattedValue(
  raw1: number,
  format: string,
  dataType: string = 'Int16',
  byteOrder: string = 'ABCD',
  raw2?: number
): { word1: number; word2?: number } {
  // Increment the numeric value independently of its display format.
  void format;
  if (dataType === 'Coil') return { word1: raw1 === 1 ? 0 : 1 };
  const current = Number(formatRegisterValue(raw1, 'Dec', dataType, byteOrder, raw2));
  const min = dataType === 'Int32' ? -2147483648 : dataType === 'Int16' ? -32768 : 0;
  const max = dataType === 'Int32' ? 2147483647 : dataType === 'UInt32' ? 4294967295 : dataType === 'Int16' ? 32767 : 65535;
  const next = dataType === 'Float32' ? current + 1 : current >= max ? min : current + 1;
  const result = parseFormattedRegisterValue(String(next), 'Dec', dataType, byteOrder);
  return typeof result === 'number' ? { word1: result } : result;
}
