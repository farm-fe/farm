import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createCompiler, resolveConfig, getSortedPluginHooksBindThis } from '@farmfe/core';
import { summarizeGraph } from './graph-summary.mjs';
import { captureNativeProvenance } from './native-provenance.mjs';
import { captureSourceProvenance } from './source-provenance.mjs';
import { instrumentPluginHooks } from './hook-diagnostics.mjs';
import { scaleProfile } from './scale-profiles.mjs';

const group = process.env.COMMERCE_GROUP ?? 'js';
const scale = process.env.COMMERCE_SCALE ?? 'baseline';
scaleProfile(scale);
assert.ok(['js', 'rust', 'mixed'].includes(group), 'Invalid group');
const identity = `graph-${Date.now()}-${group}-${scale}`;
process.env.COMMERCE_CACHE_ID = identity;
process.env.COMMERCE_OUTPUT_ID = identity;
const report = { kind: 'retained-compilation-graph', group, scale, success: false, identity };
const tracing = process.env.COMMERCE_TRACE_HOOKS === '1';
const pending = new Map();
let stage = 'provenance';
let journal;
let journalWrite = Promise.resolve();
report.diagnosticTracing = tracing;
if (tracing) {
  await fs.mkdir('reports', { recursive: true });
  journal = setInterval(() => {
    const snapshot = { identity, pid: process.pid, stage, terminal: false, pending: [...pending.values()], memory: process.memoryUsage(), observedAt: new Date().toISOString() };
    journalWrite = journalWrite.then(() => fs.writeFile(`reports/${identity}.live.json`, JSON.stringify(snapshot, null, 2))).catch(error => console.error(error.message));
  }, 1000);
  journal.unref();
}
try {
  report.nativeBefore = await captureNativeProvenance();
  report.sourceBefore = await captureSourceProvenance();
  const config = group === 'js' ? 'farm.config.ts' : `farm.${group}.config.ts`;
  stage = 'resolve-config';
  const resolved = await resolveConfig({ root: process.cwd(), configFile: path.resolve(config) }, 'build', 'production');
  assert.equal(path.resolve(resolved.configFilePath), path.resolve(config), 'Requested configuration was not loaded');
  report.configuration = { file: resolved.configFilePath,
    jsPlugins: resolved.jsPlugins.map(plugin => plugin.name), rustPlugins: resolved.rustPlugins };
  if (tracing) instrumentPluginHooks(resolved.jsPlugins, pending);
  stage = 'configure-compiler';
  const compiler = createCompiler(resolved);
  for (const hook of getSortedPluginHooksBindThis(resolved.jsPlugins ?? [], 'configureCompiler')) await hook?.(compiler);
  stage = 'native-compile';
  await compiler.compile();
  stage = 'trace-graph';
  compiler.writeResourcesToDisk();
  const graph = await compiler.traceModuleGraph();
  report.graph = graph;
  report.summary = summarizeGraph(graph);
  report.resources = Object.entries(compiler.resources()).map(([name, bytes]) => ({ name, bytes: bytes.length })).sort((a, b) => a.name.localeCompare(b.name));
  report.resourceCount = report.resources.length;
  report.resourceBytes = report.resources.reduce((total, resource) => total + resource.bytes, 0);
  report.workloadSourceFiles = report.sourceBefore.workload.files.length;
  report.workloadSourceBytes = report.sourceBefore.workload.files.reduce((total, file) => total + file.bytes, 0);
  report.nativeAfter = await captureNativeProvenance();
  report.sourceAfter = await captureSourceProvenance();
  assert.deepEqual(report.nativeAfter, report.nativeBefore);
  assert.deepEqual(report.sourceAfter, report.sourceBefore);
  report.success = true;
} catch (error) { report.error = error.stack; process.exitCode = 1; }
finally {
  if (journal) {
    clearInterval(journal);
    await journalWrite;
    await fs.writeFile(`reports/${identity}.live.json`, JSON.stringify({ identity, pid: process.pid, stage, terminal: true, success: report.success, error: report.error, pending: [...pending.values()], observedAt: new Date().toISOString() }, null, 2));
  }
}
await fs.mkdir('reports', { recursive: true });
await fs.writeFile(`reports/${identity}.json`, JSON.stringify(report, null, 2));
console.log(JSON.stringify({ group, success: report.success, summary: report.summary, resourceCount: report.resourceCount, error: report.error }));
