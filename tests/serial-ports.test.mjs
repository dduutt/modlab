import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import ts from 'typescript';
import { computed, effectScope, nextTick, reactive, ref, watch } from 'vue';
import { loadTs } from './helpers.mjs';
const { connectionError } = await loadTs('../src/utils/configValidation.ts');

const component = readFileSync(new URL('../src/components/SettingsModal.vue', import.meta.url), 'utf8');
const script = component.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const source = ts.createSourceFile('SettingsModal.ts', script, ts.ScriptTarget.Latest, true);
const setup = source.statements.filter(statement =>
  !ts.isImportDeclaration(statement) && !ts.isInterfaceDeclaration(statement),
).map(statement => statement.getText(source)).join('\n');
const compiled = ts.transpileModule(setup, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;
const createSetup = new Function('ref', 'watch', 'computed', 'onUnmounted', 'defineProps', 'defineEmits', 'ModbusService', 'connectionError', 'setTimeout', 'clearTimeout',
  `${compiled}\nreturn { form, serialPorts, portsLoading, portsRefreshing, portsError, canSave, refreshSerialPorts, handleSave };`);

function modal(t, listSerialPorts, serialPort = 'COM4') {
  const scope = effectScope();
  const props = reactive({ show: true, config: { protocol: 'RTU', serialPort,
    ip: '127.0.0.1', port: 502, baudRate: 9600, dataBits: 8, stopBits: 1, parity: 'None', timeoutMs: 1000, retries: 3 } });
  const emitted = [];
  const timers = new Map();
  let timerId = 0;
  let unmount;
  const state = scope.run(() => createSetup(ref, watch, computed, callback => { unmount = callback; },
    () => props, () => (...event) => emitted.push(event), { listSerialPorts }, connectionError,
    (callback, delay) => { timers.set(++timerId, { callback, delay }); return timerId; }, id => timers.delete(id)));
  t.after(() => { unmount(); scope.stop(); });
  const tickRefresh = () => {
    assert.equal(timers.size, 1);
    const [id, timer] = [...timers][0];
    assert.equal(timer.delay, 1000);
    timers.delete(id);
    return timer.callback();
  };
  return { ...state, props, emitted, timers, tickRefresh, unmount: () => unmount() };
}

test('lists only detected ports, naturally sorted, and preserves a valid selection', async t => {
  const state = modal(t, async () => ['COM10', 'COM4', 'COM2', 'COM4']);
  assert.equal(state.portsLoading.value, true);
  assert.equal(state.canSave.value, false);
  await nextTick();
  assert.deepEqual(state.serialPorts.value, ['COM2', 'COM4', 'COM10']);
  assert.equal(state.form.value.serialPort, 'COM4');
  state.handleSave();
  assert.equal(state.emitted[0][1].serialPort, 'COM4');
});

test('an absent saved port defaults to the first detected port in natural order', async t => {
  const state = modal(t, async () => ['COM10', 'COM2'], 'COM99');
  await nextTick();
  assert.equal(state.form.value.serialPort, 'COM2');
  state.handleSave();
  assert.equal(state.emitted[0][1].serialPort, 'COM2');
  assert.equal(state.canSave.value, true);
});

test('no ports blocks RTU save but does not block TCP settings', async t => {
  const state = modal(t, async () => []);
  await nextTick();
  assert.deepEqual(state.serialPorts.value, []);
  assert.equal(state.form.value.serialPort, '');
  assert.equal(state.canSave.value, false);
  state.form.value.protocol = 'TCP';
  state.handleSave();
  assert.equal(state.emitted[0][0], 'save');
});

test('refresh removes unplugged ports and can recover from discovery failure', async t => {
  let result = ['COM4'];
  const state = modal(t, async () => {
    if (result instanceof Error) throw result;
    return result;
  });
  await nextTick();
  result = new Error('Discovery failed');
  await state.refreshSerialPorts();
  assert.deepEqual(state.serialPorts.value, []);
  assert.equal(state.portsError.value, 'Discovery failed');
  assert.equal(state.canSave.value, false);
  result = ['COM5'];
  await state.refreshSerialPorts();
  assert.deepEqual(state.serialPorts.value, ['COM5']);
  assert.equal(state.form.value.serialPort, 'COM5');
  assert.equal(state.portsError.value, '');
});

test('late discovery from a closed modal cannot overwrite a newer list', async t => {
  const pending = [];
  const state = modal(t, () => new Promise(resolve => pending.push(resolve)));
  state.props.show = false;
  await nextTick();
  state.props.show = true;
  await nextTick();
  assert.equal(pending.length, 2);
  pending[1](['COM4']);
  await nextTick();
  pending[0](['COM99']);
  await nextTick();
  assert.deepEqual(state.serialPorts.value, ['COM4']);
  assert.equal(state.form.value.serialPort, 'COM4');
});

test('two stop bits survive save and reopening, and invalid input cannot emit save', async t => {
  const state = modal(t, async () => ['COM4']);
  await nextTick();
  state.form.value.stopBits = 2;
  state.handleSave();
  const saved = state.emitted[0][1];
  assert.equal(saved.stopBits, 2);
  state.props.show = false;
  await nextTick();
  state.props.config = saved;
  state.props.show = true;
  await nextTick();
  assert.equal(state.form.value.stopBits, 2);
  state.form.value.timeoutMs = -1;
  const before = state.emitted.length;
  state.handleSave();
  assert.equal(state.emitted.length, before);
});

test('background refresh adds ports without clearing the list, selection or save availability', async t => {
  let resolve;
  let calls = 0;
  const state = modal(t, () => ++calls === 1 ? Promise.resolve(['COM4']) : new Promise(done => { resolve = done; }));
  await nextTick();
  const pending = state.tickRefresh();
  assert.equal(state.portsRefreshing.value, true);
  assert.equal(state.portsLoading.value, false);
  assert.equal(state.canSave.value, true);
  assert.deepEqual(state.serialPorts.value, ['COM4']);
  await state.refreshSerialPorts();
  assert.equal(calls, 2, 'manual refresh cannot overlap a running scan');
  assert.equal(state.timers.size, 0, 'next scan waits for completion');
  resolve(['COM10', 'COM4', 'COM2']);
  await pending;
  assert.deepEqual(state.serialPorts.value, ['COM2', 'COM4', 'COM10']);
  assert.equal(state.form.value.serialPort, 'COM4');
  assert.equal(state.timers.size, 1);
});

test('hot-unplug clears the selection without selecting another device on later scans', async t => {
  let ports = ['COM4', 'COM5'];
  const state = modal(t, async () => ports);
  await nextTick();
  ports = ['COM5'];
  await state.tickRefresh();
  assert.deepEqual(state.serialPorts.value, ['COM5']);
  assert.equal(state.form.value.serialPort, '');
  assert.equal(state.canSave.value, false);
  await state.tickRefresh();
  assert.equal(state.form.value.serialPort, '');
  state.form.value.serialPort = 'COM5';
  assert.equal(state.canSave.value, true);
});

test('automatic refresh recovers after transient discovery failures', async t => {
  let failure = false;
  const state = modal(t, async () => {
    if (failure) throw new Error('Temporary discovery failure');
    return ['COM4'];
  });
  await nextTick();
  const previousList = state.serialPorts.value;
  failure = true;
  await state.tickRefresh();
  assert.equal(state.form.value.serialPort, 'COM4');
  assert.equal(state.canSave.value, false);
  assert.equal(state.timers.size, 1);
  failure = false;
  await state.tickRefresh();
  assert.equal(state.portsError.value, '');
  assert.equal(state.canSave.value, true);
  assert.equal(state.serialPorts.value, previousList, 'unchanged results do not replace the options');
});

test('closing, switching to TCP and unmounting cancel scheduled refreshes', async t => {
  const state = modal(t, async () => ['COM4']);
  await nextTick();
  assert.equal(state.timers.size, 1);
  state.props.show = false;
  await nextTick();
  assert.equal(state.timers.size, 0);
  state.props.show = true;
  await nextTick();
  assert.equal(state.timers.size, 1);
  state.form.value.protocol = 'TCP';
  await nextTick();
  assert.equal(state.timers.size, 0);
  state.form.value.protocol = 'RTU';
  await nextTick();
  assert.equal(state.timers.size, 1);
  state.unmount();
  assert.equal(state.timers.size, 0);
});

test('closing during a background scan prevents late results and timer restart', async t => {
  let resolve;
  let calls = 0;
  const state = modal(t, () => ++calls === 1 ? Promise.resolve(['COM4']) : new Promise(done => { resolve = done; }));
  await nextTick();
  const pending = state.tickRefresh();
  state.props.show = false;
  await nextTick();
  resolve(['COM99']);
  await pending;
  assert.deepEqual(state.serialPorts.value, ['COM4']);
  assert.equal(state.portsRefreshing.value, false);
  assert.equal(state.timers.size, 0);
});
