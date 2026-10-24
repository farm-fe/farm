import ts from 'typescript';
import path from 'node:path';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { captureSourceProvenance } from './source-provenance.mjs';

export function analyzeComponentSources(sources, roots) {
  const files = new Map(Object.entries(sources).filter(([file]) => !file.startsWith('packages/extensions/')).map(([file, content]) => [file, ts.createSourceFile(file, content, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX)]));
  function resolve(file, source) {
    if (!source.startsWith('.')) return null;
    const base = path.posix.normalize(path.posix.join(path.posix.dirname(file), source));
    return [base, `${base}.tsx`, `${base}.ts`, `${base}/index.tsx`, `${base}/index.ts`].find(candidate => files.has(candidate)) ?? null;
  }
  function visit(node, callback) { callback(node); ts.forEachChild(node, child => visit(child, callback)); }
  const reachable = new Set();
  function follow(file) {
    if (reachable.has(file)) return;
    if (!files.has(file)) throw new Error(`Missing component inventory root: ${file}`);
    reachable.add(file);
    visit(files.get(file), node => {
      let source;
      if (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) source = node.moduleSpecifier;
      if (ts.isCallExpression(node) && node.expression.kind === ts.SyntaxKind.ImportKeyword) source = node.arguments[0];
      if (source && ts.isStringLiteral(source)) { const target = resolve(file, source.text); if (target) follow(target); }
    });
  }
  roots.forEach(follow);
  const components = [];
  function hasJsx(node) { let result = false; visit(node, child => { if (ts.isJsxElement(child) || ts.isJsxFragment(child) || ts.isJsxSelfClosingElement(child)) result = true; }); return result; }
  for (const file of [...reachable].sort()) for (const statement of files.get(file).statements) {
    const definitions = [];
    if ((ts.isFunctionDeclaration(statement) || ts.isClassDeclaration(statement)) && statement.name) definitions.push({ name: statement.name.text, node: statement });
    if (ts.isVariableStatement(statement)) for (const declaration of statement.declarationList.declarations) if (ts.isIdentifier(declaration.name) && declaration.initializer && (ts.isArrowFunction(declaration.initializer) || ts.isFunctionExpression(declaration.initializer))) definitions.push({ name: declaration.name.text, node: declaration.initializer });
    for (const definition of definitions) if (/^[A-Z]/.test(definition.name) && hasJsx(definition.node)) {
      const isDefault = statement.modifiers?.some(modifier => modifier.kind === ts.SyntaxKind.DefaultKeyword) ?? false;
      const kind = file.includes('/pages/') ? 'page' : /\/(main|store)\.tsx$/.test(file) ? 'infrastructure' : 'business-shared';
      components.push({ file, name: definition.name, kind, defaultExport: isDefault, referenced: kind === 'page' && isDefault });
    }
  }
  const byId = new Map(components.map(component => [`${component.file}#${component.name}`, component]));
  for (const file of reachable) {
    const imports = new Map();
    for (const statement of files.get(file).statements) if (ts.isImportDeclaration(statement) && ts.isStringLiteral(statement.moduleSpecifier)) {
      const target = resolve(file, statement.moduleSpecifier.text); if (!target) continue;
      const clause = statement.importClause;
      if (clause?.name) { const component = components.find(component => component.file === target && component.defaultExport); if (component) imports.set(clause.name.text, `${target}#${component.name}`); }
      if (clause?.namedBindings && ts.isNamedImports(clause.namedBindings)) for (const item of clause.namedBindings.elements) imports.set(item.name.text, `${target}#${item.propertyName?.text ?? item.name.text}`);
    }
    visit(files.get(file), node => {
      if ((ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) && ts.isIdentifier(node.tagName)) {
        const id = imports.get(node.tagName.text) ?? `${file}#${node.tagName.text}`;
        const component = byId.get(id); if (component) component.referenced = true;
      }
    });
  }
  const count = kind => components.filter(component => component.kind === kind && component.referenced).length;
  return { manualBusinessSharedCount: count('business-shared'), pageCount: count('page'), infrastructureCount: count('infrastructure'), components, reachableFiles: [...reachable].sort(), limitation: 'Static JSX reference audit for supplied TS/TSX source. Pages and infrastructure are separate; generated extensions, MDX/SVG outputs and dependencies are excluded. References do not prove runtime coverage, semantic complexity or performance.' };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const before = await captureSourceProvenance();
  const sources = {};
  async function walk(directory) { for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const file = path.posix.join(directory, entry.name);
    if (entry.isDirectory()) await walk(file);
    else if (/\.tsx?$/.test(file) && !file.endsWith('.d.ts')) sources[file] = await fs.readFile(file, 'utf8');
  } }
  await walk('apps/admin'); await walk('packages/ui');
  const report = analyzeComponentSources(sources, ['apps/admin/main.tsx']);
  report.sourceBefore = before;
  report.sourceAfter = await captureSourceProvenance();
  if (JSON.stringify(report.sourceBefore) !== JSON.stringify(report.sourceAfter)) throw new Error('Source changed during component inventory');
  await fs.mkdir('reports', { recursive: true });
  const file = `reports/components-${Date.now()}.json`;
  await fs.writeFile(file, JSON.stringify(report, null, 2));
  console.log(JSON.stringify({ file, manualBusinessSharedCount: report.manualBusinessSharedCount, pageCount: report.pageCount, infrastructureCount: report.infrastructureCount }));
}
