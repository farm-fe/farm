import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';
import { execFileSync } from 'node:child_process';

const npmCli = process.env.npm_execpath ?? path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');

const results = [];
const expectProbe = process.env.COMMERCE_SDK_EXPECT_PROBE === 'present';
const probeConsumption = expectProbe
  ? `const probe: string | undefined = products[0].cacheUpdateProbe;
// @ts-expect-error The updated optional field must not become any or a number.
const invalidProbe: number = products[0].cacheUpdateProbe;`
  : `// @ts-expect-error The temporary field must disappear after source restoration.
const missingProbe = products[0].cacheUpdateProbe;`;
for (const group of ['js', 'rust']) {
  const result = { group, success: false, timestamp: new Date().toISOString() };
  try {
    const directory = await fs.mkdtemp(path.join(os.tmpdir(), `commerce-sdk-${group}-`));
    result.consumerDirectory = directory;
    const installed = path.join(directory, 'node_modules/@commerce-lab/sdk');
    const artifact = path.resolve(`dist/sdk-${group}`);
    await fs.access(path.join(artifact, 'src/index.d.ts'));
    await fs.writeFile(path.join(artifact, 'package.json'), JSON.stringify({ name: '@commerce-lab/sdk', version: '0.0.1', private: true, type: 'module', main: './index.js', types: './src/index.d.ts', exports: { '.': { types: './src/index.d.ts', import: './index.js' } }, files: ['index.js', 'src/**/*.d.ts'] }, null, 2));
    const [packed] = JSON.parse(execFileSync(process.execPath, [npmCli, 'pack', '--json', '--pack-destination', directory], { cwd: artifact, encoding: 'utf8' }));
    result.packFiles = packed.files.map(file => file.path);
    await fs.writeFile(path.join(directory, 'package.json'), '{"private":true,"type":"module"}');
    execFileSync(process.execPath, [npmCli, 'install', '--ignore-scripts', '--no-audit', '--no-fund', path.join(directory, packed.filename)], { cwd: directory, encoding: 'utf8' });
    const consumer = path.join(directory, 'consumer.ts');
    await fs.writeFile(consumer, `import { quoteCampaign, type CampaignProduct, type CampaignQuote } from '@commerce-lab/sdk';
const products: CampaignProduct[] = [{ id: 'p1', tenantId: 'north', price: 1999, stock: 3 }];
const result: CampaignQuote = quoteCampaign('north', products, 15);
const cents: number = result.items[0].salePrice;
${probeConsumption}
// @ts-expect-error The merchant ID must remain a string.
quoteCampaign(123, products, 15);
// @ts-expect-error Sale price must remain numeric rather than any.
const invalid: string = result.items[0].salePrice;
export { cents };
`);
    const program = ts.createProgram([consumer], { noEmit: true, strict: true, skipLibCheck: false, types: [], target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext });
    const diagnostics = ts.getPreEmitDiagnostics(program);
    assert.equal(diagnostics.length, 0, ts.formatDiagnosticsWithColorAndContext(diagnostics, { getCanonicalFileName: x => x, getCurrentDirectory: () => directory, getNewLine: () => '\n' }));
    const runtime = await import(pathToFileURL(path.join(installed, 'index.js')).href);
    assert.equal(runtime.quoteCampaign('north', [{ id: 'p1', tenantId: 'north', price: 1999, stock: 3 }], 15).items[0].salePrice, 1699);
    result.success = true;
  } catch (error) { result.error = error.message; }
  results.push(result);
  console.log(JSON.stringify(result));
}
await fs.mkdir('reports', { recursive: true });
const report = JSON.stringify({ kind: 'npm-pack-install-consumption', results }, null, 2) + '\n';
await fs.writeFile(`reports/sdk-consumer-${Date.now()}.json`, report);
await fs.writeFile('reports/sdk-consumer.json', report);
if (results.some(result => !result.success)) process.exitCode = 1;
