import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createCompiler, resolveConfig } from '@farmfe/core';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-component-declarations-'));
const declaration = path.join(temporary, 'components.d.ts');
const report = { kind: 'rust-react-components-generated-declarations', success: false, temporary,
  scope: 'Actual local component declarations with real source imports; project-compatible noImplicitAny=false, other strict flags enabled, skipLibCheck=false', runtime: process.version };
try {
  report.sourceBefore = await captureSourceProvenance();
  report.nativeBefore = await captureNativeProvenance();
  const resolved = await resolveConfig({ root: process.cwd(), configFile: path.resolve('farm.rust.config.ts') }, 'build', 'production');
  const entry = resolved.rustPlugins.find(([filename]) => filename.includes('react-components'));
  assert.ok(entry, 'Component plugin must be in the actual Rust configuration');
  const options = JSON.parse(entry[1]);
  options.dts = declaration;
  entry[1] = JSON.stringify(options);
  resolved.compilation.output.path = path.join(temporary, 'dist').replaceAll('\\', '/');
  resolved.compilation.persistentCache = false;
  const compiler = createCompiler(resolved);
  await compiler.compile();
  const generated = await fs.readFile(declaration, 'utf8');
  assert.match(generated, /const ProductImageMetadata: typeof import/);
  report.declaration = declaration;
  report.declarationCount = (generated.match(/\bconst \w+:/g) ?? []).length;
  await fs.symlink(path.resolve('node_modules'), path.join(temporary, 'node_modules'), 'junction');
  // Shared ambient asset declarations remain necessary, but the target component
  // must obtain its type from the plugin's generated file, not a handwritten const.
  const ambient = (await fs.readFile('apps/admin/env.d.ts', 'utf8'))
    .split('\n').filter(line => !line.startsWith('declare const ProductImageMetadata:')).join('\n');
  await fs.writeFile(path.join(temporary, 'assets.d.ts'), ambient);
  await fs.writeFile(path.join(temporary, 'consumer.tsx'),
    "const valid = <ProductImageMetadata category='Home' name='Canvas'/>;\n// @ts-expect-error category is a string, not an unchecked any\nconst invalid = <ProductImageMetadata category={123} name='Canvas'/>;\nexport { valid, invalid };\n");
  await fs.writeFile(path.join(temporary, 'tsconfig.json'), JSON.stringify({ compilerOptions: {
    noEmit: true, strict: true, noImplicitAny: false, skipLibCheck: false,
    module: 'ESNext', moduleResolution: 'Bundler', target: 'ES2022', jsx: 'react-jsx',
    allowJs: true, checkJs: false, esModuleInterop: true,
    types: ['react', 'react-dom', 'node'],
    typeRoots: [path.resolve('node_modules/@types'), path.resolve('../../node_modules/@types')],
    baseUrl: temporary,
    paths: {
      react: [path.resolve('node_modules/@types/react/index.d.ts')],
      'react/*': [path.resolve('node_modules/@types/react/*')],
      'react-dom': [path.resolve('node_modules/@types/react-dom/index.d.ts')],
      'react-dom/*': [path.resolve('node_modules/@types/react-dom/*')],
    },
  }, include: ['consumer.tsx', 'assets.d.ts', 'components.d.ts'] }));
  const runTypes = () => execFileSync(process.execPath, [path.resolve('node_modules/typescript/bin/tsc'), '-p', path.join(temporary, 'tsconfig.json')],
    { windowsHide: true, encoding: 'utf8', timeout: 120000 });
  try { report.typesOutput = runTypes(); }
  catch (error) {
    // Reproduce dependency errors without importing any generated declarations.
    await fs.writeFile(path.join(temporary, 'direct-consumer.ts'),
      `import { ProductBulkActions } from ${JSON.stringify(path.resolve('apps/admin/components/ProductBulkActions').replaceAll('\\', '/'))};\nexport type BulkProps = Parameters<typeof ProductBulkActions>[0];\n`);
    const config = JSON.parse(await fs.readFile(path.join(temporary, 'tsconfig.json'), 'utf8'));
    config.include = ['direct-consumer.ts', 'assets.d.ts'];
    await fs.writeFile(path.join(temporary, 'direct-tsconfig.json'), JSON.stringify(config));
    try {
      execFileSync(process.execPath, [path.resolve('node_modules/typescript/bin/tsc'), '-p', path.join(temporary, 'direct-tsconfig.json')],
        { windowsHide: true, encoding: 'utf8', timeout: 120000 });
      report.directLibraryBaselinePassed = true;
    } catch (baseline) {
      report.directLibraryBaselinePassed = false;
      report.directLibraryErrors = baseline.stdout?.toString();
      const errors = output => output.split(/\r?\n/).filter(line => line.includes('error TS')).sort();
      report.sameErrorsWithoutGeneratedDeclarations = JSON.stringify(errors(error.stdout.toString())) === JSON.stringify(errors(report.directLibraryErrors));
    }
    throw error;
  }
  report.sourceAfter = await captureSourceProvenance();
  report.nativeAfter = await captureNativeProvenance();
  assert.deepEqual(report.sourceAfter, report.sourceBefore);
  assert.deepEqual(report.nativeAfter, report.nativeBefore);
  report.success = true;
} catch (error) {
  report.error = error.stack;
  report.compilerOutput = error.stdout?.toString();
  process.exitCode = 1;
} finally {
  report.sourceAfter ??= await captureSourceProvenance();
  report.nativeAfter ??= await captureNativeProvenance();
  report.identitiesUnchanged = JSON.stringify(report.sourceAfter) === JSON.stringify(report.sourceBefore)
    && JSON.stringify(report.nativeAfter) === JSON.stringify(report.nativeBefore);
  const filename = `reports/component-declarations-${Date.now()}.json`;
  await writeJsonCheckpoint(filename, report);
  console.log(JSON.stringify({ filename, success: report.success, declarationCount: report.declarationCount,
    error: report.error, compilerErrors: report.compilerOutput?.split('\n').filter(line => line.includes('error TS')).slice(0, 8) }));
}
