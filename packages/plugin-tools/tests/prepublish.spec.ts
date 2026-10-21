import { readFileSync } from 'node:fs';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it, vi } from 'vitest';

const calls = vi.hoisted(() => vi.fn());
vi.mock('child_process', () => ({ execSync: calls }));

import { prepublish } from '../src/prepublish.js';

it('prepares libc and versions before any platform publication', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-prepublish-'));
  const previous = process.cwd();
  const observed: string[] = [];
  try {
    await fs.writeFile(
      path.join(root, 'package.json'),
      JSON.stringify({
        name: '@farmfe/plugin-probe',
        version: '2.0.0',
        files: ['scripts']
      })
    );
    for (const abi of ['linux-arm64-musl', 'linux-x64-gnu']) {
      const directory = path.join(root, 'npm', abi);
      await fs.mkdir(directory, { recursive: true });
      await fs.writeFile(
        path.join(directory, 'package.json'),
        JSON.stringify({
          name: `@farmfe/plugin-probe-${abi}`,
          version: '0.0.0',
          os: ['linux'],
          cpu: [abi.includes('arm64') ? 'arm64' : 'x64'],
          libc: ['glibc'],
          files: ['index.farm']
        })
      );
    }
    calls.mockImplementation((command, options) => {
      expect(command).toBe('npm publish --access public');
      const manifest = JSON.parse(
        readFileSync(path.join(options.cwd, 'package.json'), 'utf8')
      );
      expect(manifest.version).toBe('2.0.0');
      expect(manifest.libc).toEqual([
        options.cwd.endsWith('-musl') ? 'musl' : 'glibc'
      ]);
      observed.push(manifest.name);
      return '';
    });
    process.chdir(root);
    await prepublish();
    expect(observed).toHaveLength(2);
    const parent = JSON.parse(
      await fs.readFile(path.join(root, 'package.json'), 'utf8')
    );
    expect(Object.values(parent.optionalDependencies)).toEqual([
      '2.0.0',
      '2.0.0'
    ]);
  } finally {
    process.chdir(previous);
    calls.mockReset();
    await fs.rm(root, { recursive: true, force: true });
  }
});
