import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { instrumentPluginHooks } from './hook-diagnostics.mjs';

export function createStartupDiagnosticPlugin(pending, observe) {
  return {
    name: 'commerce-startup-diagnostics',
    configResolved(config) {
      instrumentPluginHooks(config.jsPlugins ?? [], pending);
      observe('config-resolved', config);
    },
    configureServer(server) { observe('server-created', server); },
    configureCompiler(compiler) { observe('compiler-created', compiler); },
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { Server } = await import('@farmfe/core');
  const { captureNativeProvenance } = await import('./native-provenance.mjs');
  const { captureSourceProvenance } = await import('./source-provenance.mjs');
  const group = process.env.COMMERCE_GROUP ?? 'mixed';
  assert.ok(['js', 'rust', 'mixed'].includes(group), 'Invalid diagnostic group');
  const identity = `startup-diagnostic-${Date.now()}-${group}`;
  process.env.COMMERCE_CACHE_ID ??= `startup-diagnostic-${group}`;
  process.env.COMMERCE_PORT ??= '4340';
  const configFile = path.resolve(group === 'js' ? 'farm.config.ts' : `farm.${group}.config.ts`);
  const pending = new Map();
  const report = { identity, group, pid: process.pid, success: false, terminal: false, diagnosticTracing: true,
    cacheId: process.env.COMMERCE_CACHE_ID, scope: 'Real Server startup with wrapped JS hooks; instrumentation changes execution and cache identity. Not performance evidence.' };
  let stage = 'provenance', compiler, server, journalError;
  let writing = Promise.resolve();
  await fs.mkdir('reports', { recursive: true });
  const file = `reports/${identity}.json`;
  function observe(next, value) {
    stage = next;
    if (next === 'config-resolved') report.configuration = { file: value.configFilePath, jsPlugins: value.jsPlugins.map(plugin => plugin.name), rustPlugins: value.rustPlugins };
    if (next === 'compiler-created') compiler = value;
    if (next === 'server-created') server = value;
  }
  const journal = setInterval(() => {
    const snapshot = { identity, group, pid: process.pid, terminal: false, stage: compiler?.compiling ? 'native-compile' : stage,
      compiling: compiler?.compiling, pending: [...pending.values()], memory: process.memoryUsage(), observedAt: new Date().toISOString() };
    writing = writing.then(() => fs.writeFile(file.replace('.json', '.live.json'), JSON.stringify(snapshot, null, 2))).catch(error => { journalError = error.message; });
  }, 1000);
  journal.unref();
  try {
    report.nativeBefore = await captureNativeProvenance();
    report.sourceBefore = await captureSourceProvenance();
    stage = 'resolve-and-create-server';
    server = await Server.createServer({ root: process.cwd(), configFile, plugins: [createStartupDiagnosticPlugin(pending, observe)] });
    assert.equal(path.resolve(server.config.configFilePath), configFile);
    assert.ok(report.configuration, 'Diagnostic plugin was not installed before compiler creation');
    stage = 'listen-and-compile';
    await server.listen();
    assert.equal(server.getCompiler().compiling, false);
    report.success = true;
  } catch (error) { report.error = error.stack; process.exitCode = 1; }
  finally {
    stage = 'closing';
    if (server) try { await server.close(); } catch (error) { report.cleanupError = error.message; report.success = false; process.exitCode = 1; }
    clearInterval(journal);
    await writing;
    if (journalError) { report.journalError = journalError; report.success = false; process.exitCode = 1; }
    report.nativeAfter = await captureNativeProvenance();
    report.sourceAfter = await captureSourceProvenance();
    report.identitiesUnchanged = JSON.stringify(report.nativeBefore) === JSON.stringify(report.nativeAfter) && JSON.stringify(report.sourceBefore) === JSON.stringify(report.sourceAfter);
    if (!report.identitiesUnchanged) { report.success = false; process.exitCode = 1; }
    report.terminal = true;
    report.pending = [...pending.values()];
    await fs.writeFile(file, JSON.stringify(report, null, 2));
    await fs.writeFile(file.replace('.json', '.live.json'), JSON.stringify({ identity, group, pid: process.pid, terminal: true, success: report.success, pending: report.pending, error: report.error, observedAt: new Date().toISOString() }, null, 2));
    console.log(JSON.stringify({ file, success: report.success, identitiesUnchanged: report.identitiesUnchanged, pending: report.pending, error: report.error, cleanupError: report.cleanupError }));
  }
}
