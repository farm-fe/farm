import fs from 'node:fs/promises';
import path from 'node:path';
import { createCompiler, resolveConfig, getSortedPluginHooksBindThis } from '@farmfe/core';
import { summarizeGraph } from '../examples/commerce-lab/benchmark/graph-summary.mjs';
// Separate, unscored compilation after timing; never infer modules from files.
const [lab, output] = process.argv.slice(2);
if (!lab || !output || process.env.COMMERCE_SCALE !== 'business-2048') throw new Error('Explicit business-2048 graph workload required');
const resolved = await resolveConfig({ root: lab, configFile: path.join(lab, 'farm.rust.config.ts') }, 'build', 'production');
const compiler = createCompiler(resolved);
for (const hook of getSortedPluginHooksBindThis(resolved.jsPlugins ?? [], 'configureCompiler')) await hook?.(compiler);
await compiler.compile(); compiler.writeResourcesToDisk();
const summary = summarizeGraph(await compiler.traceModuleGraph());
await fs.writeFile(output, JSON.stringify({ status: 'complete', scale: 'business-2048', measured: false, summary }));
