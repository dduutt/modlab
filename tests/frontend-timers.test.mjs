import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import ts from 'typescript';
import { loadTs } from './helpers.mjs';

const { incrementFormattedValue } = await loadTs('../src/utils/modbusFormatter.ts');
const script = readFileSync(new URL('../src/App.vue', import.meta.url), 'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const source = ts.createSourceFile('App.ts', script, ts.ScriptTarget.Latest, true);
const names = new Set(['logFailure', 'captureTabRequest', 'clearTabTimers', 'updateTabTimers', 'handleConfirmCloseTab']);
const extracted = source.statements.filter(node => ts.isFunctionDeclaration(node) && names.has(node.name?.text)).map(node => node.getText(source)).join('\n');
const code = ts.transpileModule(extracted, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
const create = new Function('setInterval', 'clearInterval', 'readTabRegisters', 'writeTabRegister', 'ModbusService', 'incrementFormattedValue', 'tabs', 'tabToCloseId', 'showConfirmModal', 'handleCloseTab', `
  const tabTimers = new Map(), pollingInFlight = new Set(), incrementInFlight = new Set();
  const broadcastLog = () => {};
  ${code}
  return { updateTabTimers, clearTabTimers, handleConfirmCloseTab };
`);

function fixture(role = 'Master') {
  const tab = { statusMessage: '', statusType: 'info', id: 'test', title: 'Test', connected: true, isPolling: true, autoIncrement: true,
    config: { unitId: 1, startAddress: 0, count: 2, functionCode: '0x03', interval: 100, dataType: 'Int16', format: 'Dec', byteOrder: 'ABCD' },
    connection: { role, timeoutMs: 1000, retries: 0 }, values: {} };
  const callbacks = [];
  const service = { read: async () => ({ ...tab.values }) };
  const closed = [];
  const handlers = create(callback => { callbacks.push(callback); return callbacks.length; }, () => {},
    () => service.read(), (...args) => service.write(...args), service, incrementFormattedValue,
    { value: [tab] }, { value: tab.id }, { value: true }, id => closed.push(id));
  return { tab, callbacks, service, closed, ...handlers };
}

test('slow polling never overlaps and stopped polling cannot apply a late response', async () => {
  const f = fixture();
  let resolve;
  let calls = 0;
  f.service.read = () => { calls++; return new Promise(done => { resolve = done; }); };
  f.updateTabTimers(f.tab);
  const pending = f.callbacks[0]();
  await f.callbacks[0]();
  assert.equal(calls, 1);
  f.clearTabTimers(f.tab.id);
  resolve({ 0: 123 });
  await pending;
  assert.deepEqual(f.tab.values, {});
  f.updateTabTimers(f.tab);
  const resumed = f.callbacks[1]();
  resolve({ 0: 456 });
  await resumed;
  assert.deepEqual(f.tab.values, { 0: 456 });
});

test('coil increment ignores a previously selected 32-bit data type and cannot overlap', async () => {
  const f = fixture('Slave');
  f.tab.config.functionCode = '0x01';
  f.tab.config.dataType = 'Float32';
  f.tab.values = { 0: 0, 1: 1 };
  const writes = [];
  let resolve;
  f.service.write = async (_tab, address, value) => {
    writes.push([address, value]);
    if (writes.length === 1) await new Promise(done => { resolve = done; });
  };
  f.updateTabTimers(f.tab);
  const pending = f.callbacks[0]();
  await f.callbacks[0]();
  assert.equal(writes.length, 1);
  resolve();
  await pending;
  assert.deepEqual(writes, [[0, 1], [1, 0]]);
  assert.deepEqual(f.tab.values, { 0: 1, 1: 0 });
});

test('failed disconnect keeps the tab available and releases busy state', async () => {
  const f = fixture();
  f.service.disconnect = async () => { throw new Error('Disconnect unavailable'); };
  await f.handleConfirmCloseTab();
  assert.deepEqual(f.closed, []);
  assert.equal(f.tab.connected, true);
  assert.equal(f.tab.busy, false);
  assert.match(f.tab.statusMessage, /Disconnect unavailable/);
});

for (const protocol of ['TCP', 'RTU']) {
  test(`${protocol} polling replaces a previous error after a successful slave response`, async () => {
    const f = fixture();
    f.tab.connection.protocol = protocol;
    let failed = true;
    f.service.read = async () => {
      if (failed) throw new Error('Read timed out');
      return { 0: 123, 1: 456 };
    };
    f.updateTabTimers(f.tab);
    await f.callbacks[0]();
    assert.equal(f.tab.statusType, 'error');
    assert.match(f.tab.statusMessage, /Read timed out/);
    failed = false;
    await f.callbacks[0]();
    assert.equal(f.tab.statusType, 'success');
    assert.match(f.tab.statusMessage, /Communication restored/);
    assert.doesNotMatch(f.tab.statusMessage, /timed out/);
    assert.deepEqual(f.tab.values, { 0: 123, 1: 456 });
  });
}

test('auto increment preserves a failed write error and clears it only after a successful cycle', async () => {
  const f = fixture('Slave');
  let failed = true;
  f.service.write = async () => {
    if (failed) throw new Error('Write failed');
  };
  f.updateTabTimers(f.tab);
  await f.callbacks[0]();
  assert.equal(f.tab.statusType, 'error');
  assert.match(f.tab.statusMessage, /Write failed/);
  failed = false;
  await f.callbacks[0]();
  assert.equal(f.tab.statusType, 'info');
  assert.match(f.tab.statusMessage, /Listening/);
});

test('slave grid syncs external writes without reporting local reads as wire success', async () => {
  const f = fixture('Slave');
  f.tab.autoIncrement = false;
  f.tab.statusMessage = 'Listening';
  f.tab.statusType = 'info';
  f.service.read = async () => ({ 0: 321, 1: 654 });
  f.updateTabTimers(f.tab);
  await f.callbacks[0]();
  assert.deepEqual(f.tab.values, { 0: 321, 1: 654 });
  assert.equal(f.tab.statusMessage, 'Listening');
  assert.equal(f.tab.statusType, 'info');
  let resolve;
  f.service.read = () => new Promise(done => { resolve = done; });
  const pending = f.callbacks[0]();
  f.clearTabTimers(f.tab.id);
  f.tab.connected = false;
  resolve({ 0: 999 });
  await pending;
  assert.deepEqual(f.tab.values, { 0: 321, 1: 654 });
});

test('a late slave snapshot cannot overwrite a confirmed local edit', async () => {
  const f = fixture('Slave');
  f.tab.autoIncrement = false;
  let resolve;
  f.service.read = () => new Promise(done => { resolve = done; });
  f.updateTabTimers(f.tab);
  const pending = f.callbacks[0]();
  f.tab.values[0] = 99;
  f.tab.valueVersion = 1;
  resolve({ 0: 10 });
  await pending;
  assert.equal(f.tab.values[0], 99);
});

test('auto increment uses current backend memory rather than an outdated displayed value', async () => {
  const f = fixture('Slave');
  f.tab.values = { 0: 1, 1: 2 };
  f.service.read = async () => ({ 0: 100, 1: 200 });
  const writes = [];
  f.service.write = async (_tab, address, value) => { writes.push([address, value]); };
  f.updateTabTimers(f.tab);
  await f.callbacks[0]();
  assert.deepEqual(writes, [[0, 101], [1, 201]]);
  assert.deepEqual(f.tab.values, { 0: 101, 1: 201 });
});

for (const role of ['Master', 'Slave']) {
  test(`${role} background success stays quiet and preserves unrelated errors`, async () => {
    const f = fixture(role);
    f.service.write = async () => {};
    f.tab.statusType = 'info';
    f.tab.statusMessage = 'Listening';
    f.updateTabTimers(f.tab);
    await f.callbacks[0]();
    assert.equal(f.tab.statusMessage, 'Listening');
    f.tab.statusType = 'error';
    f.tab.statusMessage = '[Test] Disconnect failed: busy';
    await f.callbacks[0]();
    assert.equal(f.tab.statusType, 'error');
    assert.equal(f.tab.statusMessage, '[Test] Disconnect failed: busy');
  });
}
