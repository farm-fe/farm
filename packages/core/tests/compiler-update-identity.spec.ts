import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it, vi } from 'vitest';
import { Compiler } from '../src/compiler/index.js';

const native = vi.hoisted(() => ({
  known: new Set<string>(),
  realpathErrors: new Map<string, string>(),
  calls: [] as unknown[]
}));
vi.mock('node:fs', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs')>();
  return {
    ...actual,
    realpathSync: (file: string) => {
      const code = native.realpathErrors.get(file);
      if (code) throw Object.assign(new Error(code), { code });
      return actual.realpathSync(file);
    }
  };
});
vi.mock('../binding/index.js', () => ({
  Compiler: class {
    hasModule(id: string) {
      return native.known.has(id);
    }
    async update(paths: unknown, finish: () => Promise<void>) {
      native.calls.push(paths);
      await finish();
      return {};
    }
  }
}));

it.each([true, false])(
  'preserves membership, unknown and query identities (symlinks=%s)',
  async (symlinks) => {
    native.calls = [];
    native.known.clear();
    native.realpathErrors.clear();
    const owner = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-update-identity-')
    );
    try {
      const file = path.join(owner, 'module.js');
      const alias = path.join(owner, 'alias.js');
      await fs.writeFile(file, 'export default 1');
      await fs.symlink(file, alias);
      const physical = await fs.realpath(file);
      // Match native hasModule's filename lookup (including query-only modules).
      native.known.add(physical);
      const unknown = path.join(owner, 'unknown.js');
      await fs.writeFile(unknown, 'export default 2');
      const compiler = new Compiler({
        compilation: { root: owner, resolve: { symlinks } }
      } as any);
      const items = [
        { path: alias + '?variant', type: 'updated' as const },
        { path: alias, type: 'added' as const },
        { path: alias, type: 'removed' as const },
        { path: unknown, type: 'updated' as const },
        { path: 'virtual:probe', type: 'updated' as const }
      ];
      await compiler.update(items);
      expect(native.calls).toEqual([
        items.map((item, index) => [
          index === 0 && symlinks ? physical + '?variant' : item.path,
          item.type
        ])
      ]);
      // An explicitly existing lexical module takes precedence over a physical alias.
      native.known.add(alias);
      await compiler.update([items[0]]);
      expect(native.calls[1]).toEqual([[alias + '?variant', 'updated']]);
    } finally {
      await fs.rm(owner, { recursive: true, force: true });
    }
  }
);

it.each(['ENOENT', 'ENOTDIR', 'EACCES'])(
  'only falls back for disappearing realpath targets (%s)',
  async (code) => {
    native.calls = [];
    native.known.clear();
    native.realpathErrors.clear();
    const owner = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-alias-race-'));
    try {
      const file = path.join(owner, 'event.js');
      const sibling = path.join(owner, 'sibling.js');
      await fs.writeFile(file, 'export default 1');
      await fs.writeFile(sibling, 'export default 2');
      native.known.add(sibling);
      native.realpathErrors.set(file, code);
      const compiler = new Compiler({ compilation: { root: owner } } as any);
      const items = [
        { path: file, type: 'updated' as const },
        { path: sibling, type: 'updated' as const }
      ];
      if (code === 'EACCES') {
        await expect(compiler.update(items)).rejects.toThrow('EACCES');
        expect(native.calls).toHaveLength(0);
      } else {
        await compiler.update(items);
        expect(native.calls).toEqual([
          items.map((item) => [item.path, item.type])
        ]);
      }
      expect(compiler.compiling).toBe(false);
    } finally {
      native.realpathErrors.clear();
      await fs.rm(owner, { recursive: true, force: true });
    }
  }
);
