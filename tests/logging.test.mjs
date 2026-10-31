import assert from 'node:assert/strict';
import test from 'node:test';
import { loadTs } from './helpers.mjs';

const { appendLog, decodeFrame, formatSummary } = await loadTs('../src/utils/logging.ts');

function trimLogs(session) {
  session.logs = session.logs.slice(0, 2000);
}

function broadcastLog(session, direction, message, bytes, detail = false, startedAt) {
  const timestampMs = Date.now();
  const level = /failed/i.test(message) ? 'error' : /restored/i.test(message) ? 'success' : 'info';
  const logEntry = {
    origin: 'frontend',
    id: crypto.randomUUID(),
    sessionId: session.id,
    sessionTitle: session.title,
    direction,
    message,
    bytes,
    timestampMs,
    durationMs: startedAt === undefined ? undefined : Math.max(0, timestampMs - startedAt),
    timestamp:
      new Date(timestampMs).toLocaleTimeString('en-GB', { hour12: false }) +
      '.' +
      String(timestampMs % 1000).padStart(3, '0'),
    kind: 'operation',
    level,
    detail: (detail || direction === 'TX') && level !== 'error',
  };
  if (level !== 'error' || !/read|write|connect|disconnect/i.test(message)) return;
  session.logs = appendLog(session.logs, {
    ...logEntry,
    time: logEntry.timestamp,
    deviceName: session.title,
  });
  trimLogs(session);
}

function logFailure(session, operation, error, startedAt) {
  broadcastLog(
    session,
    'RX',
    `${operation} Failed`,
    error instanceof Error ? error.message : String(error),
    false,
    startedAt
  );
}

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

test('formatSummary returns friendly summary for frames and operation logs', () => {
  // Operation log
  assert.equal(formatSummary({ kind: 'operation', message: 'Connection lost' }), 'Connection lost');

  // TCP Read Holding Registers Request
  assert.equal(
    formatSummary({
      kind: 'frame',
      protocol: 'TCP',
      role: 'Master',
      direction: 'TX',
      bytes: '00 01 00 00 00 06 01 03 00 00 00 0A',
    }),
    'Unit 1 · Read Holding Registers · Addr: 0, Count: 10'
  );

  // TCP Read Holding Registers Response (10 registers = 20 bytes)
  assert.equal(
    formatSummary({
      kind: 'frame',
      protocol: 'TCP',
      role: 'Master',
      direction: 'RX',
      bytes: '00 01 00 00 00 17 01 03 14 00 01 00 02',
    }),
    'Unit 1 · Read Holding Registers · Resp: 10 reg(s)'
  );

  // RTU Write Single Register Request
  assert.equal(
    formatSummary({
      kind: 'frame',
      protocol: 'RTU',
      role: 'Master',
      direction: 'TX',
      bytes: '01 06 00 05 04 D2 19 6F',
    }),
    'Unit 1 · Write Single Register · Addr: 5, Val: 1234'
  );

  // Modbus Exception Response
  assert.equal(
    formatSummary({
      kind: 'frame',
      protocol: 'TCP',
      role: 'Master',
      direction: 'RX',
      bytes: '00 01 00 00 00 03 01 83 02',
    }),
    'Unit 1 · FC 0x03 Exception 0x02'
  );
});
