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
      return Number.isNaN(val) ? 'NaN' : parseFloat(val.toFixed(4)).toString();
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

  if (dataType === 'Float32' || dataType === 'Int32' || dataType === 'UInt32') {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);

    if (dataType === 'Float32') {
      const floatVal = parseFloat(trimmed) || 0;
      view.setFloat32(0, floatVal, false);
    } else if (dataType === 'Int32') {
      const rawInt = parseInt(trimmed.replace(/^0x/i, ''), format === 'Hex' ? 16 : 10) || 0;
      const clampedInt = Math.max(-2147483648, Math.min(2147483647, rawInt));
      view.setInt32(0, clampedInt, false);
    } else {
      const rawUint = parseInt(trimmed.replace(/^0x/i, ''), format === 'Hex' ? 16 : 10) || 0;
      const clampedUint = Math.max(0, Math.min(4294967295, rawUint));
      view.setUint32(0, clampedUint, false);
    }

    const bytes = new Uint8Array(buffer);
    const unswapped = unswapBytes32(bytes, byteOrder);
    const word1 = (unswapped[0] << 8) | unswapped[1];
    const word2 = (unswapped[2] << 8) | unswapped[3];

    return { word1, word2 };
  }

  if (format === 'Hex' || trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    const parsedHex = parseInt(trimmed.replace(/^0x/i, ''), 16) || 0;
    return parsedHex & 0xffff;
  }

  if (dataType === 'Int16') {
    const val = parseInt(trimmed, 10) || 0;
    const clamped = Math.max(-32768, Math.min(32767, val));
    return clamped < 0 ? clamped + 65536 : clamped;
  }

  const val = parseInt(trimmed, 10) || 0;
  return Math.max(0, Math.min(65535, val));
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
  const currentStr = formatRegisterValue(raw1, format, dataType, byteOrder, raw2);

  if (dataType === 'Coil') {
    const currentVal = parseInt(currentStr, 10) || 0;
    const nextVal = currentVal === 1 ? 0 : 1;
    return { word1: nextVal };
  }

  if (dataType === 'Float32') {
    const currentVal = parseFloat(currentStr) || 0;
    const nextVal = (currentVal + 1).toFixed(4);
    const parsed = parseFormattedRegisterValue(nextVal, format, dataType, byteOrder);
    if (typeof parsed === 'object') {
      return { word1: parsed.word1, word2: parsed.word2 };
    }
  }

  if (dataType === 'Int32' || dataType === 'UInt32') {
    const isHex = format === 'Hex' || currentStr.startsWith('0x') || currentStr.startsWith('0X');
    const currentVal = parseInt(currentStr.replace(/^0x/i, ''), isHex ? 16 : 10) || 0;
    const nextVal = currentVal + 1;
    const nextStr = isHex ? '0x' + nextVal.toString(16) : nextVal.toString(10);
    const parsed = parseFormattedRegisterValue(nextStr, format, dataType, byteOrder);
    if (typeof parsed === 'object') {
      return { word1: parsed.word1, word2: parsed.word2 };
    }
  }

  const isHex = format === 'Hex' || currentStr.startsWith('0x') || currentStr.startsWith('0X');
  const currentVal = parseInt(currentStr.replace(/^0x/i, ''), isHex ? 16 : 10) || 0;
  const nextVal = currentVal + 1;
  const nextStr = isHex ? '0x' + nextVal.toString(16) : nextVal.toString(10);
  const parsed = parseFormattedRegisterValue(nextStr, format, dataType, byteOrder);
  return { word1: typeof parsed === 'number' ? parsed & 0xffff : 0 };
}

