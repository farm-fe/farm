import { execFileSync } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const cacheId = process.env.COMMERCE_SDK_CACHE_ID ?? `sdk-acceptance-${Date.now()}`;
if (!/^[a-zA-Z0-9_-]+$/.test(cacheId)) throw new Error('Invalid SDK cache ID');
const report = { kind: 'sdk-cache-consumption', cacheId, startedAt: new Date().toISOString(), phases: [] };
let failed = false;
try {
  for (const phase of ['initial', 'warm', 'warm-repeat']) {
    const result = { phase, success: false, builds: [] };
    report.phases.push(result);
    for (const group of ['js', 'rust']) {
      const output = execFileSync(process.execPath, [path.resolve(root, '../../packages/cli/bin/farm.mjs'), 'build', '--config', `farm.sdk-${group}.config.ts`], {
        cwd: root, encoding: 'utf8', timeout: 120000,
        env: { ...process.env, COMMERCE_SDK_CACHE_ID: cacheId },
      });
      result.builds.push({ group, output });
    }
    execFileSync(process.execPath, ['tests/sdk-consumer.mjs'], { cwd: root, encoding: 'utf8', timeout: 120000 });
    result.consumption = JSON.parse(await fs.readFile(path.join(root, 'reports/sdk-consumer.json'), 'utf8'));
    result.success = result.consumption.results.every(item => item.success);
    console.log(JSON.stringify({ phase, cacheId, success: result.success }));
    if (!result.success) throw new Error(`SDK consumption failed in ${phase}`);
  }
} catch (error) {
  failed = true;
  report.error = { message: error.message, stdout: error.stdout?.toString(), stderr: error.stderr?.toString() };
  console.error(error.message);
} finally {
  report.finishedAt = new Date().toISOString();
  report.success = !failed;
  await fs.mkdir(path.join(root, 'reports'), { recursive: true });
  await fs.writeFile(path.join(root, `reports/sdk-cache-${cacheId}-${Date.now()}.json`), JSON.stringify(report, null, 2));
}
if (failed) process.exitCode = 1;
