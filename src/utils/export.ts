export function exportRegistersToJSON(sessionName: string, config: any, values: Record<number, number>) {
  const data = {
    session: sessionName,
    exportedAt: new Date().toISOString(),
    config,
    registers: values,
  };
  const jsonStr = JSON.stringify(data, null, 2);
  const blob = new Blob([jsonStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${sessionName.replace(/\s+/g, '_')}_registers.json`;
  a.click();
  URL.revokeObjectURL(url);
}
