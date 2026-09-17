import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import ts from 'typescript';

// Exercise the actual setup handlers with deferred IPC responses, without
// starting a Tauri webview or replacing their implementation with a test copy.
const component = readFileSync(new URL('../src/App.vue', import.meta.url), 'utf8');
const script = component.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const source = ts.createSourceFile('App.ts', script, ts.ScriptTarget.Latest, true);
const names = new Set([
  'logFailure', 'captureTabRequest',
  'readTabRegisters', 'writeTabRegister', 'handleToggleConnect', 'handleFillRandom',
  'handleReadOnce', 'handleUpdateCell', 'handleUpdateCellPair',
]);
const functions = source.statements.filter(statement =>
  ts.isFunctionDeclaration(statement) && names.has(statement.name?.text),
).map(statement => statement.getText(source)).join('\n');
const compiled = ts.transpileModule(functions, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;
const createHandlers = new Function(
  'activeTab', 'ModbusService', 'broadcastLog', 'clearTabTimers', 'generateRandomRegisters', 'console',
  `${compiled}\nreturn { ${[...names].join(', ')} };`,
);

function createTab(id) {
  return {
    id, title: id, connected: true, values: {}, logs: [], statusType: 'info', statusMessage: '',
    config: { startAddress: 10, count: 2, functionCode: '0x03', unitId: 1, dataType: 'Int16', byteOrder: 'ABCD' },
    connection: { role: 'Master', protocol: 'TCP', ip: '127.0.0.1', port: 502, timeoutMs: 1000, retries: 0 },
  };
}

for (const connected of [false, true]) {
  test(`${connected ? 'disconnect' : 'connect'} prevents duplicate requests and releases busy state on failure`, async () => {
    const tab = createTab('A');
    tab.connected = connected;
    let reject;
    const pending = new Promise((_, no) => { reject = no; });
    let calls = 0;
    const invoke = async () => { calls++; return pending; };
    const handlers = createHandlers({ value: tab }, { connect: invoke, disconnect: invoke },
      () => {}, () => {}, () => ({}), { error() {} });
    const action = handlers.handleToggleConnect();
    assert.equal(tab.busy, true);
    await handlers.handleToggleConnect();
    assert.equal(calls, 1);
    reject(new Error('Connection failed'));
    await action;
    assert.equal(tab.busy, false);
    assert.equal(tab.connected, connected);
    assert.equal(tab.statusType, 'error');
  });
}

const cases = [
  { name: 'single write', handler: 'handleUpdateCell', args: [10, 123], result: undefined, values: { 10: 123 } },
  { name: 'pair write', handler: 'handleUpdateCellPair', args: [10, 123, 11, 456], result: undefined, values: { 10: 123, 11: 456 } },
  { name: 'read', handler: 'handleReadOnce', args: [], result: { 10: 321 }, values: { 10: 321 } },
  { name: 'random fill', handler: 'handleFillRandom', args: [], result: undefined, values: { 10: 42, 11: 43 } },
  { name: 'connect', handler: 'handleToggleConnect', args: [], result: 'Connected', initiallyConnected: false, connected: true },
  { name: 'disconnect', handler: 'handleToggleConnect', args: [], result: 'Disconnected', connected: false },
];

for (const scenario of cases) {
  for (const fail of [false, true]) {
    test(`${scenario.name}: ${fail ? 'failure' : 'success'} stays with the originating tab`, async () => {
      const first = createTab('A');
      first.statusType = 'error';
      first.statusMessage = '[A] Previous timeout.';
      first.connected = scenario.initiallyConnected ?? true;
      const second = createTab('B');
      const unchanged = structuredClone(second);
      const activeTab = { value: first };
      let resolve;
      let reject;
      const pending = new Promise((yes, no) => { resolve = yes; reject = no; });
      const calls = [];
      const mock = async (...args) => {
        calls.push(args);
        return calls.length === 1 ? pending : undefined;
      };
      const service = Object.fromEntries(['connect', 'disconnect', 'readRegisters', 'writeRegister', 'writeRegisters'].map(name => [name, mock]));
      const handlers = createHandlers(activeTab, service,
        (tab, ...entry) => tab.logs.push(entry), () => {}, () => ({ 10: 42, 11: 43 }), { error() {} });
      const action = handlers[scenario.handler](...scenario.args);
      assert.equal(calls.length, 1, 'IPC should be in flight before switching tabs');
      activeTab.value = second;
      if (fail) reject(new Error('Device unavailable'));
      else resolve(scenario.result);
      await action;

      assert.deepEqual(second, unchanged);
      assert.ok(calls.every(([sessionId]) => sessionId === 'A'));
      if (fail) {
        assert.equal(first.statusType, 'error');
        assert.match(first.statusMessage, /Device unavailable/);
        assert.deepEqual(first.values, {});
      } else {
        assert.equal(first.statusType, scenario.name === 'disconnect' ? 'info' : 'success');
        assert.doesNotMatch(first.statusMessage, /Previous timeout/);
        if (scenario.values) assert.deepEqual(first.values, scenario.values);
        if ('connected' in scenario) assert.equal(first.connected, scenario.connected);
        assert.ok(first.logs.length >= 2);
      }
    });
  }
}

for (const scenario of cases.filter(item => item.values)) {
  for (const change of ['unit', 'area', 'reconnect']) {
    for (const fail of [false, true]) {
      test(`${scenario.name}: stale ${fail ? 'error' : 'response'} after ${change} cannot change the current display`, async () => {
        const tab = createTab('A');
        let finish;
        let reject;
        const pending = new Promise((yes, no) => { finish = yes; reject = no; });
        let calls = 0;
        const invoke = () => { calls++; return pending; };
        const service = Object.fromEntries(['readRegisters', 'writeRegister', 'writeRegisters'].map(name => [name, invoke]));
        const handlers = createHandlers({ value: tab }, service, () => {}, () => {}, () => ({ 10: 42, 11: 43 }), { error() {} });
        const action = handlers[scenario.handler](...scenario.args);
        if (change === 'unit') tab.config.unitId = 2;
        if (change === 'area') tab.config.functionCode = '0x04';
        if (change === 'reconnect') tab.connectionVersion = 1;
        tab.statusMessage = 'Current device status';
        tab.statusType = 'info';
        if (fail) reject(new Error('Old request failed'));
        else finish(scenario.result);
        await action;
        assert.equal(tab.statusMessage, 'Current device status');
        assert.equal(tab.statusType, 'info');
        assert.deepEqual(tab.values, {});
        assert.equal(calls, 1, 'batch writes stop when their target changes');
      });
    }
  }
}
