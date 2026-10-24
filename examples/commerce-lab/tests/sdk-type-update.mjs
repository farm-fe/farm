import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const sourcePath = path.join(root, 'packages/sdk/src/models.ts');
const original = await fs.readFile(sourcePath, 'utf8');
const anchor = '  stock: number;';
assert.equal(original.split(anchor).length, 2, 'model edit anchor must occur once');
const cacheId = `sdk-type-update-${Date.now()}`;
const report = { kind: 'sdk-type-dependency-update', cacheId, phases: [], success: false };
async function verify(phase, changed) {
  const result = { phase, success: false, builds: [] };
  report.phases.push(result);
  for (const group of ['js', 'rust']) {
    const output = execFileSync(process.execPath, [path.resolve(root, '../../packages/cli/bin/farm.mjs'), 'build', '--config', `farm.sdk-${group}.config.ts`], {
      cwd: root, encoding: 'utf8', timeout: 120000,
      env: { ...process.env, COMMERCE_SDK_CACHE_ID: cacheId },
    });
    const declaration = await fs.readFile(path.join(root, `dist/sdk-${group}/src/models.d.ts`), 'utf8');
    result.builds.push({ group, output, declaration });
    assert.equal(/cacheUpdateProbe\?:\s*string/.test(declaration), changed, `${group}: stale declaration in ${phase}`);
  }
  execFileSync(process.execPath, ['tests/sdk-consumer.mjs'], {
    cwd: root, encoding: 'utf8', timeout: 120000,
    env: { ...process.env, COMMERCE_SDK_EXPECT_PROBE: changed ? 'present' : 'absent' },
  });
  result.consumption = JSON.parse(await fs.readFile(path.join(root, 'reports/sdk-consumer.json'), 'utf8'));
  assert.ok(result.consumption.results.every(item => item.success));
  result.success = true;
  console.log(JSON.stringify({ phase, cacheId, success: true }));
}
try {
  await verify('baseline', false);
  await fs.writeFile(sourcePath, original.replace(anchor, `${anchor}\n  cacheUpdateProbe?: string;`));
  await verify('type-edit', true);
  await verify('type-edit-warm', true);
} catch (error) {
  report.error = { message: error.message, stdout: error.stdout?.toString(), stderr: error.stderr?.toString() };
  console.error(error.message);
} finally {
  await fs.writeFile(sourcePath, original);
  try {
    await verify('restored', false);
    await verify('restored-warm', false);
    report.success = !report.error;
  } catch (error) {
    report.restoreError = error.message;
    console.error(error.message);
  }
  await fs.mkdir(path.join(root, 'reports'), { recursive: true });
  await fs.writeFile(path.join(root, `reports/sdk-type-update-${Date.now()}.json`), JSON.stringify(report, null, 2));
}
if (!report.success) process.exitCode = 1;
