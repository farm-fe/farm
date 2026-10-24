import { readdir } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// These two suites require built workspace/native artifacts; retain them in
// npm test and run them again as part of complete candidate acceptance.
const artifactSuites = new Set([
  'rust-plugin-purity.test.mjs',
  'source-provenance.test.mjs',
]);
const root = fileURLToPath(new URL('../', import.meta.url));
const directory = new URL('../tests/', import.meta.url);
const files = (await readdir(directory))
  .filter(name => name.endsWith('.test.mjs') && !artifactSuites.has(name))
  .sort()
  .map(name => fileURLToPath(new URL(name, directory)));
const result = spawnSync(process.execPath, ['--test', ...files], {
  cwd: root,
  stdio: 'inherit',
  windowsHide: true,
});
if (result.error) console.error(result.error);
process.exitCode = result.status ?? 1;
