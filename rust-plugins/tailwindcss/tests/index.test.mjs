import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const packageDir = dirname(dirname(fileURLToPath(import.meta.url)));
const localBinary = join(packageDir, 'npm/linux-x64-gnu/index.farm');
let createdLocalBinary = false;

before(async () => {
  if (!existsSync(localBinary)) {
    await writeFile(localBinary, '');
    createdLocalBinary = true;
  }
});

after(async () => {
  if (createdLocalBinary) {
    await rm(localBinary, { force: true });
  }
});

test('loads only tailwind.config.js from the current working directory', async () => {
  const projectDir = await mkdtemp(join(tmpdir(), 'farm-tailwind-'));
  const previousCwd = process.cwd();

  await writeFile(
    join(projectDir, 'tailwind.config.js'),
    'module.exports = { theme: { extend: { colors: { brand: "#123456" } } } };\n'
  );
  await writeFile(
    join(projectDir, 'tailwind.config.ts'),
    'export default { theme: { extend: { colors: { ignored: "#654321" } } } };\n'
  );

  try {
    process.chdir(projectDir);
    const { default: tailwindcss } = await import('../index.js');
    const [binPath, options] = tailwindcss();

    assert.equal(resolve(binPath), resolve(localBinary));
    assert.deepEqual(options, {
      config: {
        theme: {
          extend: {
            colors: {
              brand: '#123456'
            }
          }
        }
      }
    });
  } finally {
    process.chdir(previousCwd);
    await rm(projectDir, { recursive: true, force: true });
  }
});
