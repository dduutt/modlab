import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import ts from 'typescript';
import { loadTs } from './helpers.mjs';
const { appendLog, decodeFrame } = await loadTs('../src/utils/logging.ts');
const script = readFileSync(new URL('../src/App.vue', import.meta.url), 'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const source = ts.createSourceFile('App.ts', script, ts.ScriptTarget.Latest, true);
const names = new Set(['trimLogs', 'broadcastLog', 'logFailure']);
const extracted = source.statements.filter(node => ts.isFunctionDeclaration(node) && names.has(node.name?.text)).map(node => node.getText(source)).join('\n');
const code = ts.transpileModule(extracted, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
const { broadcastLog, logFailure } = new Function('tauriEmit', 'appendLog', `${code}; return { broadcastLog, logFailure };`)(() => Promise.resolve(), appendLog);

test('actual traffic and repeated errors remain separate; local operations do not appear as packets', () => {
  const tab = { id: 'A', title: 'A', logs: [] };
  broadcastLog(tab, 'TX', 'Read Req', 'Unit: 1');
  broadcastLog(tab, 'RX', 'Read Resp', 'Received 2 values');
  broadcastLog(tab, 'RX', 'Random Fill Resp OK', 'Updated 2 registers');
  assert.equal(tab.logs.length, 0);
  logFailure(tab, 'Read', new Error('Timeout'));
  logFailure(tab, 'Read', new Error('Timeout'));
  assert.equal(tab.logs.length, 2);
  assert.ok(tab.logs.every(log => log.kind === 'operation' && log.level === 'error'));
});

test('retention bounds storage without merging identical frames', () => {
  let logs = [];
  for (let i = 0; i < 2100; i++) logs = appendLog(logs, { id: String(i), kind: 'frame', bytes: '01 03' });
  assert.equal(logs.length, 2000);
  assert.equal(logs[0].id, '2099');
  assert.equal(logs.at(-1).id, '100');
});

test('RTU decode preserves raw CRC and interprets master/slave directions consistently', () => {
  const bytes = '01 03 00 00 00 02 C4 0B';
  const log = { kind: 'frame', protocol: 'RTU', complete: true, role: 'Master', direction: 'TX', bytes };
  const master = decodeFrame(log);
  assert.ok(master.includes('Address: 0'));
  assert.ok(master.includes('Count: 2'));
  assert.ok(master.includes('CRC: OK'));
  assert.deepEqual(decodeFrame({ ...log, role: 'Slave', direction: 'RX' }), master);
  assert.ok(decodeFrame({ ...log, bytes: bytes.replace('C4 0B', '00 00') }).includes('CRC: Mismatch'));
  assert.equal(log.bytes, bytes);
});

test('TCP decode uses MBAP transaction id and response register data', () => {
  const decoded = decodeFrame({ kind: 'frame', protocol: 'TCP', complete: true, role: 'Master', direction: 'RX',
    bytes: '01 09 00 00 00 07 02 03 04 00 01 00 02' });
  assert.ok(decoded.includes('Transaction ID: 265 · Protocol ID: 0 · Length: 7'));
  assert.ok(decoded.includes('Values: 1, 2'));
});

test('partial bytes are labelled incomplete without presenting fabricated values', () => {
  const decoded = decodeFrame({ kind: 'frame', protocol: 'TCP', complete: false, role: 'Master', direction: 'RX',
    bytes: '00 01 00 00 00 06 02 03 00' });
  assert.ok(decoded.includes('Incomplete / unframed data'));
  assert.ok(!decoded.some(line => line.startsWith('Values:')));
});
