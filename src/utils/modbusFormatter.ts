// Swaps 4 bytes based on Modbus byte order (all 4-byte Modbus permutations are self-inverse)
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
    const unswapped = swapBytes32(bytes, byteOrder);
    const word1 = (unswapped[0] << 8) | unswapped[1];
    const word2 = (unswapped[2] << 8) | unswapped[3];

    return { word1, word2 };
  }

  return value & 0xffff;
}
