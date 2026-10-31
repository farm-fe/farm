/** Linear interpolation at (n - 1) * p; input order is never mutated. */
export function quantile(values, probability) {
  if (!Number.isFinite(probability) || probability < 0 || probability > 1)
    throw new RangeError('Invalid quantile probability');
  if (!values.every(value => Number.isFinite(value)))
    throw new TypeError('Quantile values must be finite numbers');
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = (sorted.length - 1) * probability;
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  return sorted[lower] + (sorted[upper] - sorted[lower]) * (index - lower);
}

export function summarize(records) {
  const cells = new Map();
  for (const record of records) {
    if (typeof record.success !== 'boolean') throw new TypeError('Missing success status');
    if (record.success && (!Number.isFinite(record.durationMs) || record.durationMs < 0))
      throw new TypeError('Invalid successful durationMs');
    if (typeof record.group !== 'string' || typeof record.scenario !== 'string')
      throw new TypeError('Missing group or scenario');
    const dimensions = {
      group: record.group,
      scenario: record.scenario,
      scale: record.scale ?? 'baseline',
      cacheMode: record.cacheMode ?? 'unspecified',
    };
    const key = JSON.stringify(dimensions);
    if (!cells.has(key)) cells.set(key, { dimensions, records: [] });
    cells.get(key).records.push(record);
  }
  return [...cells.values()].map(({ dimensions, records: samples }) => {
    const valid = samples.filter(record => record.success);
    const values = valid.map(record => record.durationMs);
    const q1 = quantile(values, 0.25);
    const q3 = quantile(values, 0.75);
    return {
      ...dimensions,
      samples: samples.length,
      validSamples: valid.length,
      failures: samples.length - valid.length,
      failureRate: (samples.length - valid.length) / samples.length,
      sufficientSamples: valid.length >= (dimensions.scenario.startsWith('hmr') ? 30 : 10),
      durationMs: values.length ? {
        min: quantile(values, 0), q1, median: quantile(values, 0.5), q3,
        iqr: q3 - q1, p95: quantile(values, 0.95), max: quantile(values, 1),
      } : null,
    };
  });
}

export function toCsv(records, columns) {
  const escape = value => {
    const text = value === undefined || value === null ? '' :
      typeof value === 'object' ? JSON.stringify(value) : String(value);
    return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
  };
  return [columns.map(escape).join(','), ...records.map(record =>
    columns.map(column => escape(record[column])).join(','))].join('\r\n') + '\r\n';
}
