import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { assembleNativeArtifacts } from './assemble-v2-native-artifacts.mjs';
import { command } from './performance-report.mjs';
import { resolvePnpmCli, probePnpmCli } from './v2-pnpm-cli.mjs';
import { writeBuildFailure, redactDiagnostic } from './v2-release-diagnostics.mjs';

export function candidateJsBuilds(plan) {
  const initial = ['packages/cli', 'packages/core', 'packages/runtime', 'packages/runtime-plugin-hmr', 'packages/runtime-plugin-import-meta', 'packages/plugin-tools', 'packages/utils'];
  const remaining = plan.npm.primary.filter(pkg => !pkg.manifestPath.startsWith('rust-plugins/') && pkg.name !== 'create-farm')
    .map(pkg => path.dirname(pkg.manifestPath).replaceAll('\\', '/')).filter(dir => !initial.includes(dir));
  return [...initial, ...remaining.filter(dir => dir.startsWith('js-plugins/')).sort(), ...remaining.filter(dir => !dir.startsWith('js-plugins/')).sort()];
}
async function main() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const [pool, output] = process.argv.slice(2);
  if (pool === '--check-package-manager' && process.argv.length === 3) { console.log(JSON.stringify(probePnpmCli())); return; }
  assert.ok(pool && output, 'Expected native pool and assembly output');
  const pnpm = resolvePnpmCli();
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const records = assembleNativeArtifacts(root, path.resolve(pool), process.env.GITHUB_SHA, plan);
  fs.writeFileSync(path.resolve(output), JSON.stringify({ schemaVersion: 1, sourceCommit: process.env.GITHUB_SHA, count: records.length, records, published: false }, null, 2), { flag: 'wx' });
  const core = records.find(row => row.owner === '@farmfe/core' && row.platform === 'linux-x64-gnu');
  assert.ok(core && process.platform === 'linux' && process.arch === 'x64', 'Candidate JS producer requires the declared Linux GNU native cohort');
  const binding = path.join(root, 'packages/core/binding', path.basename(core.file));
  fs.copyFileSync(path.join(root, core.file), binding, fs.constants.COPYFILE_EXCL);
  const deadline = Date.now() + 25 * 60 * 1000;
  for (const directory of candidateJsBuilds(plan)) {
    const cwd = path.join(root, directory), log = path.join(path.dirname(path.resolve(output)), `${directory.replaceAll('/', '-')}.log`);
    let phase = 'js-preflight', name = directory;
    console.log('[candidate-js start] ' + JSON.stringify({ phase, directory, cwd }));
    try {
      const disk = fs.statfsSync(root);
      assert.ok(os.freemem() >= 3.4 * 1024 ** 3 && disk.bavail * disk.bsize >= 9.75 * 1024 ** 3 && Date.now() < deadline, 'Candidate JS producer resource/time guard');
      const manifest = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json')));
      name = manifest.name;
      assert.ok(manifest.scripts?.build && !/\bcargo\b|\bnapi\b|farm-plugin-tools build/.test(manifest.scripts.build), 'Unreviewed native build in JS candidate recipe');
      phase = 'js-build';
      await command(process.execPath, [pnpm, 'run', 'build'], { phase: `${phase}/${name}`, cwd, env: process.env, log, timeoutMs: Math.min(300000, deadline - Date.now()) });
      console.log('[candidate-js completed] ' + JSON.stringify({ phase, package: name, cwd, exitCode: 0 }));
    } catch (error) {
      try { writeBuildFailure({ root, phase, name, cwd, log, error }); }
      catch (diagnosticError) { console.error('[candidate-js diagnostics unavailable] ' + redactDiagnostic(diagnosticError.message)); }
      throw error;
    }
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
