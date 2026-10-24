import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import ts from 'typescript';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const npmCli = path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
const filename = `reports/sdk-dual-${Date.now()}.json`;
const cacheId = process.env.COMMERCE_SDK_CACHE_ID ?? `sdk-dual-${Date.now()}`;
assert.match(cacheId, /^[a-zA-Z0-9_-]+$/);
const phases = process.env.COMMERCE_SDK_PHASE ? [process.env.COMMERCE_SDK_PHASE] : ['initial', 'warm', 'warm-repeat'];
const expectedProbe = process.env.COMMERCE_SDK_EXPECT_PROBE === 'present';
const report = { kind: 'sdk-dual-package-consumption', success: false, runtime: process.version, cacheId, records: [],
  phases, expectedProbe,
  scope: 'JS/Rust DTS paths, ESM and CJS in the same packed package, fresh installs, strict NodeNext consumers in the listed phases. Not formal Farm packages or other platforms.' };
try {
  report.sourceBefore = await captureSourceProvenance(); report.nativeBefore = await captureNativeProvenance();
  for (const phase of phases) {
    for (const group of ['js', 'rust']) {
      const record = { group, phase, success: false, builds: [] };
      report.records.push(record);
      for (const format of ['esm', 'cjs']) {
        const output = execFileSync(process.execPath, ['../../packages/cli/bin/farm.mjs', 'build', '--config', `farm.sdk-${group}.config.ts`],
          { windowsHide: true, encoding: 'utf8', timeout: 120000, env: { ...process.env, COMMERCE_SDK_FORMAT: format, COMMERCE_SDK_CACHE_ID: cacheId } });
        record.builds.push({ format, output });
        await fs.access(`dist/sdk-${group}${format === 'cjs' ? '-cjs' : ''}/index.${format === 'cjs' ? 'cjs' : 'js'}`);
      }
      const directory = await fs.mkdtemp(path.join(os.tmpdir(), `farm-sdk-dual-${group}-`));
      const staging = path.join(directory, 'package');
      const consumer = path.join(directory, 'consumer');
      record.directory = directory;
      await fs.mkdir(staging); await fs.mkdir(consumer);
      await fs.cp(path.resolve(`dist/sdk-${group}`), path.join(staging, 'esm'), { recursive: true });
      await fs.cp(path.resolve(`dist/sdk-${group}-cjs`), path.join(staging, 'cjs'), { recursive: true });
      await fs.writeFile(path.join(staging, 'cjs/package.json'), '{"type":"commonjs"}');
      await fs.writeFile(path.join(staging, 'package.json'), JSON.stringify({ name: '@commerce-lab/sdk', version: '0.0.1', type: 'module',
        exports: { '.': { import: { types: './esm/src/index.d.ts', default: './esm/index.js' }, require: { types: './cjs/src/index.d.ts', default: './cjs/index.cjs' } } }, files: ['esm', 'cjs'] }));
      const [packed] = JSON.parse(execFileSync(process.execPath, [npmCli, 'pack', '--json', '--pack-destination', directory], { cwd: staging, windowsHide: true, encoding: 'utf8' }));
      record.packFiles = packed.files.map(file => file.path);
      await fs.writeFile(path.join(consumer, 'package.json'), '{"private":true,"type":"module"}');
      execFileSync(process.execPath, [npmCli, 'install', '--ignore-scripts', '--no-audit', '--no-fund', path.join(directory, packed.filename)], { cwd: consumer, windowsHide: true, encoding: 'utf8', timeout: 120000 });
      const source = `import { quoteCampaign, type CampaignProduct } from '@commerce-lab/sdk';
const products: CampaignProduct[] = [{ id: 'p1', tenantId: 'north', price: 1999, stock: 3 }];
const cents: number = quoteCampaign('north', products, 15).items[0].salePrice;
${expectedProbe ? `const probe: string | undefined = products[0].cacheUpdateProbe;
// @ts-expect-error The changed field is a string, not any.
const invalidProbe: number = products[0].cacheUpdateProbe;` : `// @ts-expect-error The temporary field must be absent after restoration.
const missingProbe = products[0].cacheUpdateProbe;`}
// @ts-expect-error Merchant ID must be a string.
quoteCampaign(123, products, 15);
// @ts-expect-error Price is numeric, not any.
const bad: string = quoteCampaign('north', products, 15).items[0].salePrice;
export { cents };\n`;
      const files = ['consumer.mts', 'consumer.cts'].map(file => path.join(consumer, file));
      for (const file of files) await fs.writeFile(file, source);
      const program = ts.createProgram(files, { noEmit: true, strict: true, skipLibCheck: false, types: [], target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext });
      const diagnostics = ts.getPreEmitDiagnostics(program);
      assert.equal(diagnostics.length, 0, ts.formatDiagnostics(diagnostics, { getCanonicalFileName: x => x, getCurrentDirectory: () => consumer, getNewLine: () => '\n' }));
      const require = createRequire(path.join(consumer, 'consumer.cjs'));
      // The ESM entry imports the package name, so conditional exports are exercised.
      await fs.writeFile(path.join(consumer, 'runtime.mjs'), "export * from '@commerce-lab/sdk';");
      const esm = await import(pathToFileURL(path.join(consumer, 'runtime.mjs')).href);
      const cjs = require('@commerce-lab/sdk');
      const products = [{ id: 'p1', tenantId: 'north', price: 1999, stock: 3 }];
      for (const module of [esm, cjs]) {
        assert.equal(module.quoteCampaign('north', products, 15).items[0].salePrice, 1699);
        assert.throws(() => module.quoteCampaign('south', products, 15), /merchant/);
        assert.throws(() => module.quoteCampaign('north', products, 101), /discount/);
      }
      assert.deepEqual(esm.quoteCampaign('north', products, 15), cjs.quoteCampaign('north', products, 15));
      record.success = true; record.strictTypes = ['NodeNext .mts', 'NodeNext .cts'];
      await writeJsonCheckpoint(filename, report);
      console.log(JSON.stringify({ group, phase, success: true }));
    }
  }
  report.sourceAfter = await captureSourceProvenance(); report.nativeAfter = await captureNativeProvenance();
  assert.deepEqual(report.sourceBefore, report.sourceAfter); assert.deepEqual(report.nativeBefore, report.nativeAfter);
  report.success = report.records.length === phases.length * 2 && report.records.every(record => record.success);
} catch (error) { report.error = error.stack; report.output = error.stdout?.toString(); process.exitCode = 1; }
finally { await writeJsonCheckpoint(filename, report); console.log(JSON.stringify({ filename, success: report.success, error: report.error })); }
