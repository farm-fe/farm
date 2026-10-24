import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const generated = new Set(['node_modules', 'dist', '.cache', 'reports', '.git']);
export async function fingerprintTree(root, inputs) {
  const files = new Map();
  const absoluteRoot = path.resolve(root);
  async function visit(input) {
    const absolute = path.resolve(absoluteRoot, input);
    const relative = path.relative(absoluteRoot, absolute);
    if (relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) throw new Error('Source path escapes root');
    const stat = await fs.lstat(absolute);
    if (stat.isSymbolicLink()) throw new Error(`Source symlink requires explicit provenance: ${relative}`);
    if (stat.isDirectory()) {
      for (const entry of await fs.readdir(absolute)) if (!generated.has(entry)) await visit(path.join(relative, entry));
    } else if (stat.isFile()) {
      const bytes = await fs.readFile(absolute);
      files.set(relative.replaceAll(path.sep, '/'), { path: relative.replaceAll(path.sep, '/'), bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
    }
  }
  for (const input of inputs) await visit(input);
  const ordered = [...files.values()].sort((a, b) => a.path.localeCompare(b.path, 'en'));
  const hash = createHash('sha256');
  for (const file of ordered) hash.update(`${file.path}\0${file.bytes}\0${file.sha256}\0`);
  return { sha256: hash.digest('hex'), files: ordered };
}

export async function captureSourceProvenance() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const repositoryRoot = fileURLToPath(new URL('../../../', import.meta.url));
  const names = await fs.readdir(root);
  return {
    measurement: await fingerprintTree(root, ['benchmark', 'tests/hmr-capabilities.mjs']),
    workload: await fingerprintTree(root, ['apps', 'packages']),
    configuration: await fingerprintTree(root, ['configs', 'adapters', 'scripts', 'package.json', 'index.html', ...names.filter(name => /^farm.*\.config\.[cm]?[jt]s$/.test(name) || /^tsconfig.*\.json$/.test(name))]),
    workspaceRuntime: await fingerprintTree(repositoryRoot, [
      'packages/core/dist', 'packages/core/package.json',
      'packages/cli/dist', 'packages/cli/bin', 'packages/cli/package.json',
      'packages/utils/dist', 'packages/utils/package.json',
      'packages/runtime/src', 'packages/runtime/package.json',
      'packages/runtime-plugin-hmr/dist', 'packages/runtime-plugin-hmr/package.json',
      'packages/runtime-plugin-import-meta/dist', 'packages/runtime-plugin-import-meta/package.json',
      'js-plugins/sass/build', 'js-plugins/sass/package.json',
      'js-plugins/svgr/build', 'js-plugins/svgr/package.json',
      'js-plugins/tailwindcss/dist', 'js-plugins/tailwindcss/package.json',
      'pnpm-lock.yaml',
    ]),
    limitation: 'Hashes cover selected example and workspace runtime files plus the dependency lock. They do not hash every installed transitive npm file; native and WASM input identities are recorded separately.',
  };
}
