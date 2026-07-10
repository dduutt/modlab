export type DataType = 'Int16' | 'UInt16' | 'Int32' | 'UInt32' | 'Float32'
export type Format = 'Dec' | 'Hex' | 'Bin'
export type ByteOrder = 'ABCD' | 'CDAB' | 'BADC' | 'DCBA'

/**
 * Reads a value from the Modbus register array based on the requested format.
 * @param data The raw uint16 array
 * @param index The starting index
 * @param type The data type
 * @param format The display format
 * @param byteOrder The byte order for 32-bit values
 */
export function formatRegisterValue(
  data: number[],
  index: number,
  type: string,
  format: string,
  byteOrder: string,
  functionCode: string
): string {
  if (index >= data.length || data[index] === undefined) return ''

  // For coils and discrete inputs, just show 0 or 1
  if (functionCode === '01' || functionCode === '02') {
    return data[index] === 1 ? '1' : '0'
  }

  let valueNum: number

  // 16-bit types
  if (type === 'Int16' || type === 'UInt16') {
    let raw = data[index] & 0xFFFF
    if (type === 'Int16' && (raw & 0x8000)) {
      valueNum = raw - 0x10000
    } else {
      valueNum = raw
    }
  } 
  // 32-bit types
  else {
    if (index + 1 >= data.length || data[index+1] === undefined) {
      return '---' // Not enough data to form 32-bit
    }
    const r1 = data[index] & 0xFFFF
    const r2 = data[index + 1] & 0xFFFF
    
    let combined = 0
    switch (byteOrder) {
      case 'ABCD': combined = (r1 << 16) | r2; break; // Big Endian
      case 'CDAB': combined = (r2 << 16) | r1; break; // Little Endian Byte Swap
      case 'BADC': combined = (((r1 & 0xFF) << 8 | (r1 >> 8)) << 16) | ((r2 & 0xFF) << 8 | (r2 >> 8)); break;
      case 'DCBA': combined = (((r2 & 0xFF) << 8 | (r2 >> 8)) << 16) | (((r1 & 0xFF) << 8 | (r1 >> 8))); break;
      default: combined = (r1 << 16) | r2;
    }

    // Convert to signed 32-bit to utilize JS bitwise ops correctly
    combined = combined | 0

    if (type === 'Float32') {
      const buffer = new ArrayBuffer(4)
      const intView = new Int32Array(buffer)
      const floatView = new Float32Array(buffer)
      intView[0] = combined
      valueNum = floatView[0]
      
      // Floats are typically displayed in Decimal, but if Hex/Bin is forced, we might just show raw bytes.
      // Usually Float32 + Hex isn't requested, but we'll handle standard float decimal rounding.
      if (format === 'Dec') {
        // limit to 4 decimal places for clean UI, and prevent scientific notation (e)
        return new Intl.NumberFormat('en-US', { useGrouping: false, maximumFractionDigits: 4 }).format(valueNum)
      }
    } else if (type === 'Int32') {
      valueNum = combined
    } else if (type === 'UInt32') {
      valueNum = combined >>> 0 // force unsigned
    } else {
      valueNum = data[index]
    }
  }

  // Formatting (Dec, Hex, Bin)
  if (format === 'Hex') {
    let hexStr = ''
    if (type.includes('32')) {
      hexStr = (valueNum >>> 0).toString(16).toUpperCase().padStart(8, '0')
    } else {
      // 16-bit
      const unsigned16 = valueNum < 0 ? valueNum + 0x10000 : valueNum
      hexStr = unsigned16.toString(16).toUpperCase().padStart(4, '0')
    }
    return `0x${hexStr}`
  }
  
  if (format === 'Bin') {
    if (type.includes('32')) {
      return (valueNum >>> 0).toString(2).padStart(32, '0')
    } else {
      const unsigned16 = valueNum < 0 ? valueNum + 0x10000 : valueNum
      return unsigned16.toString(2).padStart(16, '0')
    }
  }

  // Default to Dec
  return valueNum.toString()
}

/**
 * Parses user input back into 1 or 2 uint16 registers for writing
 */
export function parseUserInput(
  input: string, 
  type: string, 
  format: string, 
  byteOrder: string,
  functionCode: string
): number[] {
  const normalizedInput = input.trim()

  // Coils
  if (functionCode === '01') {
    const lowerInput = normalizedInput.toLowerCase()
    if (normalizedInput === '1' || lowerInput === 'true' || lowerInput === 'on') return [1]
    if (normalizedInput === '0' || lowerInput === 'false' || lowerInput === 'off') return [0]
    throw new Error("Coils only accept 0 or 1")
  }

  let num = 0
  
  if (format === 'Hex') {
    if (!/^(0x)?[0-9a-fA-F]+$/.test(normalizedInput)) throw new Error("Invalid hexadecimal number")
    num = parseInt(normalizedInput.replace(/^0x/i, ''), 16)
  } else if (format === 'Bin') {
    if (!/^[01]+$/.test(normalizedInput)) throw new Error("Invalid binary number")
    num = parseInt(normalizedInput, 2)
  } else {
    if (type === 'Float32') {
      if (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)$/.test(normalizedInput)) throw new Error("Invalid decimal number")
      num = parseFloat(normalizedInput)
    } else {
      if (!/^[+-]?\d+$/.test(normalizedInput)) throw new Error("Invalid integer")
      num = parseInt(normalizedInput, 10)
    }
  }

  if (isNaN(num)) throw new Error("Invalid number")

  // Strictly enforce data type boundaries to prevent silent bitwise overflow truncation
  if (format !== 'Hex' && format !== 'Bin') {
    if (type === 'Int16' && (num < -32768 || num > 32767)) throw new Error("Overflow: Int16 must be between -32768 and 32767")
    if (type === 'UInt16' && (num < 0 || num > 65535)) throw new Error("Overflow: UInt16 must be between 0 and 65535")
    if (type === 'Int32' && (num < -2147483648 || num > 2147483647)) throw new Error("Overflow: Int32 must be between -2147483648 and 2147483647")
    if (type === 'UInt32' && (num < 0 || num > 4294967295)) throw new Error("Overflow: UInt32 must be between 0 and 4294967295")
  }

  if (type === 'Float32') {
    const buffer = new ArrayBuffer(4)
    const floatView = new Float32Array(buffer)
    const intView = new Int32Array(buffer)
    floatView[0] = num
    num = intView[0]
  }

  if (type === 'Int16' || type === 'UInt16') {
    // 1 register
    return [num & 0xFFFF]
  } else {
    // 32-bit types, 2 registers
    const upper = (num >>> 16) & 0xFFFF
    const lower = num & 0xFFFF
    
    let r1 = 0, r2 = 0
    switch (byteOrder) {
      case 'ABCD': r1 = upper; r2 = lower; break;
      case 'CDAB': r1 = lower; r2 = upper; break;
      case 'BADC': r1 = ((upper & 0xFF) << 8) | (upper >> 8); r2 = ((lower & 0xFF) << 8) | (lower >> 8); break;
      case 'DCBA': r1 = ((lower & 0xFF) << 8) | (lower >> 8); r2 = ((upper & 0xFF) << 8) | (upper >> 8); break;
      default: r1 = upper; r2 = lower;
    }
    return [r1, r2]
  }
}
