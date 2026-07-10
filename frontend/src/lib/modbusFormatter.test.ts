import { describe, it, expect } from 'vitest'
import { formatRegisterValue, parseUserInput } from './modbusFormatter'

describe('modbusFormatter', () => {
  describe('parseUserInput (String -> Modbus Registers)', () => {
    it('parses Coils correctly', () => {
      expect(parseUserInput('1', 'Int16', 'Dec', 'ABCD', '01')).toEqual([1])
      expect(parseUserInput('On', 'Int16', 'Dec', 'ABCD', '01')).toEqual([1])
      expect(parseUserInput('0', 'Int16', 'Dec', 'ABCD', '01')).toEqual([0])
      expect(() => parseUserInput('2', 'Int16', 'Dec', 'ABCD', '01')).toThrowError()
    })

    it('parses Int16 and catches overflow', () => {
      expect(parseUserInput('32767', 'Int16', 'Dec', 'ABCD', '03')).toEqual([32767])
      expect(parseUserInput('-32768', 'Int16', 'Dec', 'ABCD', '03')).toEqual([32768]) // 0x8000
      expect(() => parseUserInput('32768', 'Int16', 'Dec', 'ABCD', '03')).toThrowError(/Overflow/)
    })

    it('parses UInt16 and catches overflow', () => {
      expect(parseUserInput('65535', 'UInt16', 'Dec', 'ABCD', '03')).toEqual([65535])
      expect(() => parseUserInput('-1', 'UInt16', 'Dec', 'ABCD', '03')).toThrowError(/Overflow/)
      expect(() => parseUserInput('65536', 'UInt16', 'Dec', 'ABCD', '03')).toThrowError(/Overflow/)
    })

    it('parses Float32 into correct 2 registers (ABCD)', () => {
      // 1234.5678 -> Float32 -> 0x449a522b
      // ABCD -> r1=0x449a (17562), r2=0x522b (21035)
      const res = parseUserInput('1234.5678', 'Float32', 'Dec', 'ABCD', '03')
      expect(res).toEqual([17562, 21035])
    })

    it('parses Float32 into correct 2 registers (CDAB)', () => {
      // CDAB -> r1=0x522b (21035), r2=0x449a (17562)
      const res = parseUserInput('1234.5678', 'Float32', 'Dec', 'CDAB', '03')
      expect(res).toEqual([21035, 17562])
    })

    it('parses Hex formats', () => {
      expect(parseUserInput('0x1234', 'UInt16', 'Hex', 'ABCD', '03')).toEqual([0x1234])
      expect(parseUserInput('0xFFFFFFFF', 'UInt32', 'Hex', 'ABCD', '03')).toEqual([65535, 65535])
    })

    it('rejects partial numeric input', () => {
      expect(() => parseUserInput('12abc', 'UInt16', 'Dec', 'ABCD', '03')).toThrowError(/Invalid integer/)
      expect(() => parseUserInput('0x12zz', 'UInt16', 'Hex', 'ABCD', '03')).toThrowError(/Invalid hexadecimal/)
      expect(() => parseUserInput('10102', 'UInt16', 'Bin', 'ABCD', '03')).toThrowError(/Invalid binary/)
      expect(() => parseUserInput('1.2.3', 'Float32', 'Dec', 'ABCD', '03')).toThrowError(/Invalid decimal/)
    })
  })

  describe('formatRegisterValue (Modbus Registers -> String)', () => {
    it('formats Coils (01)', () => {
      expect(formatRegisterValue([0, 1, 0], 1, 'Int16', 'Dec', 'ABCD', '01')).toBe('1')
    })

    it('formats Int16 and UInt16', () => {
      expect(formatRegisterValue([0x8000], 0, 'Int16', 'Dec', 'ABCD', '03')).toBe('-32768')
      expect(formatRegisterValue([0x8000], 0, 'UInt16', 'Dec', 'ABCD', '03')).toBe('32768')
    })

    it('formats Float32 accurately from 2 registers (ABCD)', () => {
      // ABCD: [17562, 21035] -> 1234.5678
      const display = formatRegisterValue([17562, 21035], 0, 'Float32', 'Dec', 'ABCD', '03')
      // Note: IEEE 754 precision means it will likely be '1234.5677'
      expect(display).toBe('1234.5677')
    })

    it('formats Float32 accurately from 2 registers (CDAB)', () => {
      const display = formatRegisterValue([21035, 17562], 0, 'Float32', 'Dec', 'CDAB', '03')
      expect(display).toBe('1234.5677')
    })

    it('formats Hex strings', () => {
      expect(formatRegisterValue([0x1234], 0, 'UInt16', 'Hex', 'ABCD', '03')).toBe('0x1234')
      expect(formatRegisterValue([0x1234, 0x5678], 0, 'UInt32', 'Hex', 'ABCD', '03')).toBe('0x12345678')
    })

    it('returns --- if missing registers for 32-bit', () => {
      expect(formatRegisterValue([17562], 0, 'Float32', 'Dec', 'ABCD', '03')).toBe('---')
    })
  })
})
