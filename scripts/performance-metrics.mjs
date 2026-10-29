// Build timings qualify only after graph, browser, provenance and cleanup acceptance.
export function qualifiedPerformanceCells(status, cells = []) {
  return status === 'complete' ? cells : cells.map(cell => ({ ...cell, validSamples: 0, medianMs: null, iqrMs: null }));
}
export function completePerformanceCells(cells) {
  return Array.isArray(cells) && cells.length === 2 && ['cold', 'warm'].every(mode => {
    const matching = cells.filter(cell => cell.cacheMode === mode);
    const row = matching[0];
    return matching.length === 1 && row.samples === 3 && row.validSamples === 3
      && Number.isFinite(row.medianMs) && row.medianMs > 0 && Number.isFinite(row.iqrMs) && row.iqrMs >= 0;
  });
}
