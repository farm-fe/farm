import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { generateScale } from '../benchmark/extension-generator.mjs';

const group = process.env.COMMERCE_GROUP ?? 'js';
const configs = { js: 'farm.config.ts', rust: 'farm.rust.config.ts', mixed: 'farm.mixed.config.ts' };
const command = process.argv[2];
if (!Object.hasOwn(configs, group)) {
  console.error(`Unknown COMMERCE_GROUP: ${group}`);
  process.exit(1);
}
if (!['start', 'build', 'preview'].includes(command)) {
  console.error('Expected start, build or preview');
  process.exit(1);
}
// Generate the selected deterministic workload before Farm resolves its alias.
// Ordinary builds fill missing files while preserving existing extension edits.
if (command !== 'preview') await generateScale(process.env.COMMERCE_SCALE ?? 'baseline', { preserveExisting: true });
const cli = fileURLToPath(new URL('../../../packages/cli/bin/farm.mjs', import.meta.url));
const cwd = fileURLToPath(new URL('../', import.meta.url));
const child = spawn(process.execPath, [cli, command, '--config', configs[group], ...process.argv.slice(3)], { cwd, stdio: 'inherit', windowsHide: true });
child.on('error', error => { console.error(error.message); process.exitCode = 1; });
child.on('exit', code => { process.exitCode = code ?? 1; });
