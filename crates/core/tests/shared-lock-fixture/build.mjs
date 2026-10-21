import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../../../../', import.meta.url));
const host = /^host: (.+)$/m.exec(execFileSync('rustc', ['-vV'], { encoding: 'utf8' }))[1];
execFileSync('cargo', ['build', '--release', '--manifest-path', 'Cargo.toml', '-p', 'farm-test-shared-lock-plugin', '--target', host],
  { stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: `${root}/target` }, windowsHide: true });
