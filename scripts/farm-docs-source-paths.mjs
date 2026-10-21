import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import ts from 'typescript';

const root = fileURLToPath(new URL('../', import.meta.url));
const extensions = ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs', '.json', '.css', '.scss', '.sass'];

export function relativeImports(source, filename) {
  const imports = [];
  const ast = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true);
  function visit(node) {
    const specifier = (ts.isImportDeclaration(node) || ts.isExportDeclaration(node))
      ? node.moduleSpecifier
      : ts.isCallExpression(node) && node.expression.kind === ts.SyntaxKind.ImportKeyword
        ? node.arguments[0] : undefined;
    if (specifier && ts.isStringLiteralLike(specifier) && specifier.text.startsWith('.')) {
      imports.push(specifier.text);
    }
    ts.forEachChild(node, visit);
  }
  visit(ast);
  return imports;
}

export function unresolvedImports(filename, imports, trackedFiles) {
  return imports.flatMap(specifier => {
    const clean = specifier.split(/[?#]/, 1)[0];
    const base = path.posix.normalize(path.posix.join(path.posix.dirname(filename), clean));
    const bases = [base];
    // Farm's resolver retries a missing .js import without that suffix.
    if (base.endsWith('.js')) bases.push(base.slice(0, -3));
    const candidates = bases.flatMap(candidate => [candidate, ...extensions.map(ext => candidate + ext), ...extensions.map(ext => candidate + '/index' + ext)]);
    if (candidates.some(candidate => trackedFiles.has(candidate))) return [];
    const folded = new Set(candidates.map(candidate => candidate.toLowerCase()));
    const matches = [...trackedFiles].filter(file => folded.has(file.toLowerCase()));
    return [{ filename, specifier, matches }];
  });
}

export function assertWebsiteSourcePaths() {
  // Git preserves exact spelling even when the host filesystem ignores case.
  const tracked = new Set(execFileSync('git', ['ls-files', '-z', '--', 'website'], { cwd: root, encoding: 'utf8' }).split('\0').filter(Boolean));
  const failures = [];
  let checked = 0;
  for (const filename of tracked) {
    if (!filename.startsWith('website/src/') || !/\.[cm]?[jt]sx?$/.test(filename)) continue;
    const imports = relativeImports(fs.readFileSync(path.join(root, filename), 'utf8'), filename);
    checked += imports.length;
    failures.push(...unresolvedImports(filename, imports, tracked));
  }
  assert.deepEqual(failures, [], `Website relative imports must match exact Git paths:\n${failures.map(f => `${f.filename}: ${f.specifier}${f.matches.length ? ' (Git: ' + f.matches.join(', ') + ')' : ' (missing tracked file)'}`).join('\n')}`);
  return { checked };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  console.log(assertWebsiteSourcePaths());
}
