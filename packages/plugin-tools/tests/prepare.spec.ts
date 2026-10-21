import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { preparePlatformPackages } from '../src/prepare.js';

async function fixture() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-prepare-'));
  await fs.mkdir(path.join(root, 'npm/win32-x64-msvc'), { recursive: true });
  await fs.writeFile(
    path.join(root, 'package.json'),
    JSON.stringify({
      name: '@farmfe/plugin-probe',
      version: '2.0.0',
      optionalDependencies: { existing: '^1.0.0' }
    })
  );
  await fs.writeFile(
    path.join(root, 'npm/win32-x64-msvc/package.json'),
    JSON.stringify({
      name: '@farmfe/plugin-probe-win32-x64-msvc',
      version: '0.0.0'
    })
  );
  return root;
}

describe('platform preparation without publication', () => {
  it('previews without writes and explicitly applies synchronized versions', async () => {
    const root = await fixture();
    const manifest = path.join(root, 'package.json');
    const before = await fs.readFile(manifest, 'utf8');
    const preview = await preparePlatformPackages(root);
    expect(preview.applied).toBe(false);
    expect(await fs.readFile(manifest, 'utf8')).toBe(before);
    expect(
      JSON.parse(
        await fs.readFile(
          path.join(root, 'npm/win32-x64-msvc/package.json'),
          'utf8'
        )
      ).version
    ).toBe('0.0.0');
    const result = await preparePlatformPackages(root, true);
    expect(result.applied).toBe(true);
    expect(
      JSON.parse(await fs.readFile(manifest, 'utf8')).optionalDependencies
    ).toEqual({
      existing: '^1.0.0',
      '@farmfe/plugin-probe-win32-x64-msvc': '2.0.0'
    });
    expect(
      JSON.parse(
        await fs.readFile(
          path.join(root, 'npm/win32-x64-msvc/package.json'),
          'utf8'
        )
      ).version
    ).toBe('2.0.0');
  });

  it('validates every platform before writing any manifest', async () => {
    const root = await fixture();
    await fs.mkdir(path.join(root, 'npm/zzz'), { recursive: true });
    await fs.writeFile(
      path.join(root, 'npm/zzz/package.json'),
      JSON.stringify({ name: 'wrong-package', version: '0.0.0' })
    );
    await expect(preparePlatformPackages(root, true)).rejects.toThrow(
      'Platform package name'
    );
    expect(
      JSON.parse(
        await fs.readFile(
          path.join(root, 'npm/win32-x64-msvc/package.json'),
          'utf8'
        )
      ).version
    ).toBe('0.0.0');
  });
});
