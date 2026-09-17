export function isIntegerInRange(value: unknown, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= min && value <= max;
}

export function connectionError(config: {
  protocol: string; ip: string; port: number; timeoutMs: number; retries: number;
  baudRate: number; dataBits: number; stopBits: number; parity: string;
}): string {
  if (!isIntegerInRange(config.timeoutMs, 100, 10000)) return 'modal.invalidTimeout';
  if (!isIntegerInRange(config.retries, 0, 10)) return 'modal.invalidRetries';
  if (config.protocol === 'TCP') {
    const ip = config.ip.trim();
    const ipv4 = /^\d{1,3}(\.\d{1,3}){3}$/.test(ip) && ip.split('.').every(part => Number(part) <= 255 && (part === '0' || !part.startsWith('0')));
    let ipv6 = false;
    if (ip.startsWith('[') && ip.endsWith(']')) {
      try { ipv6 = new URL(`http://${ip}`).hostname.startsWith('['); } catch { /* Invalid IPv6 literal. */ }
    }
    if (!ipv4 && !ipv6) return 'modal.invalidIp';
    if (!isIntegerInRange(config.port, 1, 65535)) return 'modal.invalidPort';
  } else if (!isIntegerInRange(config.baudRate, 1, 4000000) ||
      ![7, 8].includes(config.dataBits) || ![1, 2].includes(config.stopBits) ||
      !['None', 'Even', 'Odd'].includes(config.parity)) return 'modal.invalidSerial';
  return '';
}
