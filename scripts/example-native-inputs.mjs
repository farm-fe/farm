import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { nativeTargets } from './native-artifact-inventory.mjs';
export function exampleNativeInput(directory, abi, { platform = process.platform, arch = process.arch } = {}) {
  assert.ok(Object.values(nativeTargets).includes(abi) && abi.startsWith(`${platform}-${arch}`), 'Example native ABI does not match this host');
  const file = path.join(directory, 'npm', abi, 'index.farm');
  try { const stat = fs.lstatSync(file); return stat.isFile() && !stat.isSymbolicLink() ? { file, abi, verification: 'file-input-only; actual ABI requires native execution' } : null; }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
}
