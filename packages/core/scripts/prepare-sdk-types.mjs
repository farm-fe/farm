import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import ts from 'typescript';

export const upstream = Object.freeze({
  version: '3.5.3',
  declarationSha256: 'a7e430e32eaefba6b1ebf23a94a4562bfe7051cc83558d33ddc8ff342f3c18e9',
  licenseSha256: 'db9ba183d4cbfbaf6c27db2effddbb84b46340c33ed32a066bc0b144a3ea1298'
});
const expectedImports = [
  'config/types.d.ts',
  'types/binding.d.ts',
  'watcher/index.d.ts'
];
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
async function writeChanged(file, bytes) {
  try { if ((await readFile(file)).equals(Buffer.from(bytes))) return false; }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, bytes);
  return true;
}
async function declarationFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) throw new Error(`Unexpected declaration symlink: ${entry.name}`);
    if (entry.isDirectory() && entry.name !== 'vendor') {
      files.push(...await declarationFiles(path.join(directory, entry.name)));
    } else if (entry.isFile() && /\.d\.(?:ts|cts|mts)$/.test(entry.name)) {
      files.push(path.join(directory, entry.name));
    }
  }
  return files.sort();
}

/** Fix the declaration contract only; the installed watcher runtime is untouched. */
export async function prepareSdkTypes(root = fileURLToPath(new URL('..', import.meta.url))) {
  root = path.resolve(root);
  const require = createRequire(path.join(root, 'package.json'));
  const manifest = JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8'));
  // Keep the existing public runtime range. Build-time input is pinned by the
  // frozen lockfile and verified below, independently of that range.
  assert.equal(typeof manifest.dependencies.chokidar, 'string', 'Core must declare its watcher dependency');
  assert.equal(manifest.dependencies.anymatch, '3.1.3', 'Public Matcher types require an explicit dependency');
  const sourceManifest = require.resolve('chokidar/package.json');
  const source = path.dirname(sourceManifest);
  const chokidar = JSON.parse(await readFile(sourceManifest, 'utf8'));
  assert.equal(chokidar.name, 'chokidar');
  assert.equal(chokidar.version, upstream.version, 'Review changed upstream declarations before generating');
  assert.equal(JSON.parse(await readFile(require.resolve('anymatch/package.json'), 'utf8')).version, '3.1.3');
  const declaration = await readFile(path.join(source, 'types/index.d.ts'));
  const license = await readFile(path.join(source, 'LICENSE'));
  assert.equal(hash(declaration), upstream.declarationSha256, 'Unexpected upstream declaration bytes');
  assert.equal(hash(license), upstream.licenseSha256, 'Unexpected upstream MIT license bytes');
  const original = declaration.toString('utf8');
  const invalid = ' implements fs.FSWatcher';
  assert.equal(original.split(invalid).length, 2, 'Exactly one invalid native interface claim is expected');
  const patched = original.replace(invalid, '');
  const dist = path.join(root, 'dist');
  const vendor = path.join(dist, 'vendor/chokidar.d.ts');
  const updates = [];
  const found = [];
  // Inspect AST nodes, then replace only the module string literal. Preserve
  // every imported name and require all affected imports to remain type-only.
  for (const file of await declarationFiles(dist)) {
    const text = await readFile(file, 'utf8');
    const ast = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
    let relative = path.relative(path.dirname(file), vendor).replaceAll('\\', '/').replace(/\.d\.ts$/, '.js');
    if (!relative.startsWith('.')) relative = './' + relative;
    const edits = [];
    function visit(node) {
      if (ts.isImportTypeNode(node) && ts.isLiteralTypeNode(node.argument)
        && ts.isStringLiteral(node.argument.literal) && node.argument.literal.text === 'chokidar') {
        throw new Error(`Review unexpected import-type form in ${file}`);
      }
      if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node))
        && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier)
        && ['chokidar', relative].includes(node.moduleSpecifier.text)) {
        assert.ok(ts.isImportDeclaration(node) && node.importClause?.isTypeOnly,
          `Only type-only chokidar imports can be redirected: ${file}`);
        found.push(path.relative(dist, file).replaceAll('\\', '/'));
        if (node.moduleSpecifier.text === 'chokidar') {
          edits.push({ start: node.moduleSpecifier.getStart(ast), end: node.moduleSpecifier.end,
            replacement: JSON.stringify(relative) });
        }
      }
      ts.forEachChild(node, visit);
    }
    visit(ast);
    let output = text;
    for (const edit of edits.sort((a, b) => b.start - a.start)) {
      output = output.slice(0, edit.start) + edit.replacement + output.slice(edit.end);
    }
    if (output !== text) updates.push([file, output]);
  }
  assert.deepEqual(found.sort(), expectedImports, 'Review changed Core declaration imports');
  const provenance = {
    package: 'chokidar', version: upstream.version,
    sourceEntry: 'types/index.d.ts', sourceHomepage: chokidar.homepage,
    upstreamDeclarationSha256: upstream.declarationSha256,
    generatedDeclarationSha256: hash(Buffer.from(patched)),
    upstreamLicenseSha256: upstream.licenseSha256, license: 'MIT',
    licenseFile: '../../licenses/chokidar-3.5.3-MIT.txt',
    transformation: 'Remove exactly " implements fs.FSWatcher". All actual class members and WatchOptions remain unchanged.',
    typeDependency: { anymatch: '3.1.3' }
  };
  // Validate the whole input before writing. Stable bytes and no timestamps make
  // incremental builds and repeated packaging deterministic.
  const outputs = [
    [vendor, patched],
    [path.join(dist, 'vendor/chokidar-3.5.3.original.d.ts.txt'), declaration],
    [path.join(dist, 'vendor/chokidar.provenance.json'), JSON.stringify(provenance, null, 2) + '\n'],
    [path.join(root, 'licenses/chokidar-3.5.3-MIT.txt'), license],
    ...updates
  ];
  const changed = [];
  for (const [file, bytes] of outputs) if (await writeChanged(file, bytes)) changed.push(path.relative(root, file));
  return { inputVersion: chokidar.version, imports: found, changed, provenance };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  console.log(JSON.stringify(await prepareSdkTypes()));
}
