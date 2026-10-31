import { readFileSync } from 'node:fs';
import ts from 'typescript';

export async function loadTs(relativePath) {
  const code = ts.transpileModule(readFileSync(new URL(relativePath, import.meta.url), 'utf8'), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
  }).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(code).toString('base64')}`);
}

const { formatRegisterValue, parseFormattedRegisterValue } = await loadTs('../src/utils/modbusFormatter.ts');

export function generateRandomRegisters(
  start,
  count,
  dataType = 'Int16',
  byteOrder = 'ABCD',
  functionCode
) {
  const result = {};
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

export function incrementFormattedValue(
  raw1,
  format,
  dataType = 'Int16',
  byteOrder = 'ABCD',
  raw2
) {
  void format;
  if (dataType === 'Coil') return { word1: raw1 === 1 ? 0 : 1 };
  const current = Number(formatRegisterValue(raw1, 'Dec', dataType, byteOrder, raw2));
  const min = dataType === 'Int32' ? -2147483648 : dataType === 'Int16' ? -32768 : 0;
  const max = dataType === 'Int32' ? 2147483647 : dataType === 'UInt32' ? 4294967295 : dataType === 'Int16' ? 32767 : 65535;
  const next = dataType === 'Float32' ? current + 1 : current >= max ? min : current + 1;
  const result = parseFormattedRegisterValue(String(next), 'Dec', dataType, byteOrder);
  return typeof result === 'number' ? { word1: result } : result;
}
