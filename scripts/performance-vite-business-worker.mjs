import fs from 'node:fs/promises';
import path from 'node:path';
import { build } from 'vite';
import { viteBusinessConfig, BUSINESS_COMPARISON_PROTOCOL } from '../examples/commerce-lab/benchmark/vite-business.mjs';
import { VITE_COMPARISON_VERSION } from '../examples/commerce-lab/benchmark/vite-comparison.mjs';

const [lab, outputId, receipt, graph] = process.argv.slice(2);
if (!lab || !/^[a-zA-Z0-9_-]+$/.test(outputId) || !receipt || process.env.COMMERCE_SCALE !== 'business-2048') throw new Error('Explicit complete isolated business workload required');
const outDir = path.join(lab, 'dist/benchmark', outputId, 'vite');
await build(await viteBusinessConfig(lab, { outDir, graph }));
await fs.writeFile(receipt, JSON.stringify({ protocol: BUSINESS_COMPARISON_PROTOCOL, viteVersion: VITE_COMPARISON_VERSION, outDir,
  cache: 'Rollup production cache disabled; a second process is a repeat build, not a persistent-cache hit', status: 'complete' }));
