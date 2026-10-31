export function buildSchedule({ runId, rounds = 10, seed = 42 }) {
  if (!/^[a-zA-Z0-9_-]+$/.test(runId ?? '')) throw new Error('Invalid runId');
  if (!Number.isSafeInteger(rounds) || rounds < 1 || rounds > 1000) throw new Error('Invalid rounds');
  if (!Number.isSafeInteger(seed)) throw new Error('Invalid seed');
  let state = seed >>> 0;
  const records = [];
  for (let round = 0; round < rounds; round++) {
    const groups = ['js', 'rust', 'mixed'];
    for (let i = groups.length - 1; i > 0; i--) {
      state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
      const j = Math.floor((state / 4294967296) * (i + 1));
      [groups[i], groups[j]] = [groups[j], groups[i]];
    }
    for (const group of groups) {
      const identity = `${runId}-${round}-${group}`;
      for (const cacheMode of ['cold', 'warm']) records.push({ round, group, cacheMode, cacheId: identity, outputId: identity });
    }
  }
  return records;
}
