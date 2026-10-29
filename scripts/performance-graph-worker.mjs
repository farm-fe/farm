import fs from 'node:fs/promises';
import path from 'node:path';
import { createCompiler, resolveConfig, getSortedPluginHooksBindThis } from '@farmfe/core';
import { summarizeGraph, generatedCoverage } from '../examples/commerce-lab/benchmark/graph-summary.mjs';
// Separate, unscored compilation after timing; never infer modules from files.
const [requestedLab, output] = process.argv.slice(2);
if (!requestedLab || !output || process.env.COMMERCE_SCALE !== 'business-2048') throw new Error('Explicit business-2048 graph workload required');
const inputRoot = path.resolve(requestedLab);
const lab = await fs.realpath(inputRoot);
const generatedInputRoot = path.join(lab, 'packages/extensions/business-2048');
const generatedRoot = await fs.realpath(generatedInputRoot);
const aliases = [{ from: inputRoot, to: lab }, { from: generatedInputRoot, to: generatedRoot }]
  .filter(row => row.from !== row.to);
const observed = new Set();
const resolved = await resolveConfig({ root: lab, configFile: path.join(lab, 'farm.rust.config.ts'), plugins: [{ name: 'commerce-farm-source-coverage',
  processModule: { filters: { resolvedPaths: ['.*'] }, executor(param) {
    observed.add(param.moduleId);
    return null;
  } } }] }, 'build', 'production');
const compiler = createCompiler(resolved);
for (const hook of getSortedPluginHooksBindThis(resolved.jsPlugins ?? [], 'configureCompiler')) await hook?.(compiler);
await compiler.compile(); compiler.writeResourcesToDisk();
const graph = await compiler.traceModuleGraph();
const summary = summarizeGraph(graph, { projectRoot: lab, aliases });
const generated = [];
async function walk(directory) { for (const entry of await fs.readdir(directory, { withFileTypes: true })) { const file = path.join(directory, entry.name); if (entry.isDirectory()) await walk(file); else generated.push(path.relative(generatedRoot, file).replaceAll('\\', '/')); } }
await walk(generatedRoot);
const coverage = generatedCoverage(generated,
  { processed: [...observed], retained: graph.modules.map(module => module.id) }, generatedRoot, lab, { aliases });
await fs.writeFile(output, JSON.stringify({ status: generated.length === 28673 && !coverage.missing.length ? 'complete' : 'invalid', scale: 'business-2048', measured: false, summary,
  reason: generated.length !== 28673 ? `Unexpected generated input count: ${generated.length}` : coverage.reason,
  coverage: { ...coverage,
    processedSources: [...observed].sort(), retainedModules: graph.modules.map(module => ({ id: module.id, type: module.moduleType })),
    semantics: 'Actual processing hooks plus retained identities; generated files are not relabeled as modules. Worker/query/virtual nodes remain distinct identities. Nested native worker compiler graphs are not exposed by this parent trace.' } }));
