/** Fixed-height order rows; end is exclusive and spacers preserve scroll height. */
export function virtualWindow(count, scrollTop, viewportHeight, rowHeight, overscan = 4) {
  if (!Number.isInteger(count) || count < 0 || !Number.isFinite(scrollTop) ||
      !Number.isFinite(viewportHeight) || viewportHeight <= 0 ||
      !Number.isFinite(rowHeight) || rowHeight <= 0 ||
      !Number.isInteger(overscan) || overscan < 0) {
    throw new RangeError('Invalid virtual list geometry');
  }
  const top = Math.max(0, Math.min(scrollTop, count * rowHeight - viewportHeight));
  const start = Math.max(0, Math.floor(top / rowHeight) - overscan);
  const end = Math.min(count, Math.ceil((top + viewportHeight) / rowHeight) + overscan);
  return { start, end, before: start * rowHeight, after: (count - end) * rowHeight };
}
