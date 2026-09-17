import type { LogEntry } from '../components/TrafficLog.vue';

// Keep every actual transfer, including identical replies and retries.
export function appendLog(logs: LogEntry[], incoming: LogEntry) {
  return [incoming, ...logs].slice(0, 2000);
}

export function decodeFrame(log: LogEntry): string[] {
  if (log.kind !== 'frame') return [log.message, log.bytes].filter(Boolean);
  const bytes = log.bytes.trim().split(/\s+/).map(byte => Number.parseInt(byte, 16));
  if (!bytes.length || bytes.some(byte => !Number.isInteger(byte))) return [];
  const tcp = log.protocol === 'TCP';
  const offset = tcp ? 6 : 0;
  const lines = [`${log.protocol || ''} · ${bytes.length} bytes`];
  const word = (i: number) => bytes[i] * 256 + bytes[i + 1];
  if (tcp && bytes.length >= 6) lines.push(`Transaction ID: ${word(0)} · Protocol ID: ${word(2)} · Length: ${word(4)}`);
  if (bytes.length < offset + 2) return lines;
  const fc = bytes[offset + 1];
  lines.push(`Unit ID: ${bytes[offset]} · FC: 0x${fc.toString(16).padStart(2, '0').toUpperCase()}`);
  if (log.complete === false) { lines.push('Incomplete / unframed data'); return lines; }
  const request = log.role === 'Master' ? log.direction === 'TX' : log.direction === 'RX';
  const end = tcp ? bytes.length : bytes.length - 2;
  const pdu = offset + 2;
  if ((fc & 0x80) !== 0 && end > pdu) lines.push(`Exception: 0x${bytes[pdu].toString(16).padStart(2, '0')}`);
  else if (request && end >= pdu + 4) {
    lines.push(`Address: ${word(pdu)}`);
    if ([1, 2, 3, 4, 15, 16].includes(fc)) lines.push(`Count: ${word(pdu + 2)}`);
    else if ([5, 6].includes(fc)) lines.push(`Value: ${word(pdu + 2)}`);
    if (fc === 15 && end >= pdu + 5) {
      const bits = bytes.slice(pdu + 5, end).flatMap(byte => Array.from({ length: 8 }, (_, bit) => (byte >> bit) & 1)).slice(0, word(pdu + 2));
      lines.push(`Values: ${bits.join(', ')}`);
    }
    if (fc === 16 && end >= pdu + 5) {
      const values = [];
      for (let i = pdu + 5; i + 1 < end; i += 2) values.push(word(i));
      lines.push(`Values: ${values.join(', ')}`);
    }
  } else if (!request && [1, 2].includes(fc) && end > pdu) {
    lines.push(`Byte count: ${bytes[pdu]}`);
    lines.push(`Bits (LSB first): ${bytes.slice(pdu + 1, end).flatMap(byte => Array.from({ length: 8 }, (_, bit) => (byte >> bit) & 1)).join(', ')}`);
  } else if (!request && [3, 4].includes(fc) && end > pdu) {
    const values = [];
    for (let i = pdu + 1; i + 1 < end; i += 2) values.push(word(i));
    lines.push(`Values: ${values.join(', ')}`);
  } else if (!request && [5, 6, 15, 16].includes(fc) && end >= pdu + 4) {
    lines.push(`Address: ${word(pdu)} · ${fc >= 15 ? 'Count' : 'Value'}: ${word(pdu + 2)}`);
  }
  if (!tcp && bytes.length >= 4) {
    let crc = 0xffff;
    for (const byte of bytes.slice(0, -2)) {
      crc ^= byte;
      for (let bit = 0; bit < 8; bit++) crc = crc & 1 ? (crc >>> 1) ^ 0xa001 : crc >>> 1;
    }
    lines.push(`CRC: ${crc === bytes[end] + bytes[end + 1] * 256 ? 'OK' : 'Mismatch'}`);
  }
  if (log.message) lines.push(log.message);
  return lines;
}
