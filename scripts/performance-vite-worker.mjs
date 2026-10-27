import fs from 'node:fs/promises';
import { runComparisonBuild } from '../examples/commerce-lab/benchmark/vite-comparison.mjs';
const [tool, fixture, output] = process.argv.slice(2);
if (!['farm', 'vite'].includes(tool) || !fixture || !output) throw new Error('Expected fixed tool, isolated fixture and result path');
const result = await runComparisonBuild(tool, fixture);
await fs.writeFile(output, JSON.stringify(result));
