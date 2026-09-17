import assert from 'node:assert/strict';
import test from 'node:test';
import { loadTs } from './helpers.mjs';

const { formatRegisterValue, parseFormattedRegisterValue, incrementFormattedValue } = await loadTs('../src/utils/modbusFormatter.ts');
const { connectionError } = await loadTs('../src/utils/configValidation.ts');

for (const type of ['Int32', 'UInt32', 'Float32']) {
  for (const order of ['ABCD', 'CDAB', 'BADC', 'DCBA']) {
    test(`${type} Hex preserves all 32 bits with ${order} byte order`, () => {
      for (const [first, second] of [[0xffff, 0xffff], [0x1234, 0xabcd], [0x3f80, 0]]) {
        const display = formatRegisterValue(first, 'Hex', type, order, second);
        assert.match(display, /^0x[0-9A-F]{8}$/);
        assert.deepEqual(parseFormattedRegisterValue(display, 'Hex', type, order), { word1: first, word2: second });
      }
    });
  }
}

test('invalid edits throw instead of silently writing zero or clamped values', () => {
  for (const [input, format, type] of [['-', 'Dec', 'Int16'], ['12-3', 'Dec', 'Int16'], ['32768', 'Dec', 'Int16'], ['-1', 'Dec', 'UInt32'], ['0x10000', 'Hex', 'UInt16'], ['0xGG', 'Hex', 'Int32'], ['1e40', 'Dec', 'Float32'], ['1.2.3', 'Dec', 'Float32']]) {
    assert.throws(() => parseFormattedRegisterValue(input, format, type));
  }
});

test('Float32 decimal display supports scientific notation without truncating small values', () => {
  const words = parseFormattedRegisterValue('1e-20', 'Dec', 'Float32');
  const display = formatRegisterValue(words.word1, 'Dec', 'Float32', 'ABCD', words.word2);
  assert.notEqual(display, '0');
  assert.deepEqual(parseFormattedRegisterValue(display, 'Dec', 'Float32'), words);
});

test('increment follows numeric value in Hex mode and wraps integer limits', () => {
  const words = parseFormattedRegisterValue('1.5', 'Dec', 'Float32');
  const next = incrementFormattedValue(words.word1, 'Hex', 'Float32', 'ABCD', words.word2);
  assert.equal(formatRegisterValue(next.word1, 'Dec', 'Float32', 'ABCD', next.word2), '2.5');
  assert.deepEqual(incrementFormattedValue(32767, 'Dec', 'Int16'), { word1: 32768 });
  assert.deepEqual(incrementFormattedValue(1, 'Hex', 'Coil'), { word1: 0 });
});

test('connection settings reject empty, fractional and out-of-range values', () => {
  const config = { protocol: 'TCP', ip: '127.0.0.1', port: 502, timeoutMs: 1000, retries: 3,
    baudRate: 9600, dataBits: 8, stopBits: 1, parity: 'None' };
  assert.equal(connectionError(config), '');
  for (const change of [{ port: 0 }, { port: 65536 }, { port: 2.5 }, { ip: '' }, { ip: '999.0.0.1' },
    { timeoutMs: '' }, { timeoutMs: -1 }, { retries: 0.5 }, { retries: 11 }, { protocol: 'RTU', stopBits: 3 }]) {
    assert.notEqual(connectionError({ ...config, ...change }), '');
  }
  assert.equal(connectionError({ ...config, protocol: 'RTU', stopBits: 2 }), '');
});
