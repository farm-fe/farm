import fs from 'node:fs';

// Shared data is also consumed by the Cargo/npm release planner. Suite-only
// expansion reflects native receipt granularity, not product version changes.
const rules = JSON.parse(fs.readFileSync(new URL('./directory-impact-rules.json', import.meta.url), 'utf8'));
export const directoryImpactRules = Object.freeze(rules);
