import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../package.json', import.meta.url));
const ts = require('typescript');
const reactTypes = path.dirname(require.resolve('@types/react/package.json'));
const nodeTypes = path.dirname(require.resolve('@types/node/package.json'));
const files = [
  '../src/machine-readable.ts',
  '../../docs-theme-default/src/document-actions-data.ts',
  '../../docs-theme-default/src/document-actions.tsx'
].map((name) => fileURLToPath(new URL(name, import.meta.url)));
const options = {
  noEmit: true,
  strict: true,
  skipLibCheck: true,
  target: ts.ScriptTarget.ES2022,
  module: ts.ModuleKind.NodeNext,
  moduleResolution: ts.ModuleResolutionKind.NodeNext,
  jsx: ts.JsxEmit.ReactJSX,
  lib: ['lib.es2022.d.ts', 'lib.dom.d.ts'],
  types: ['node', 'react'],
  typeRoots: [path.dirname(nodeTypes), path.dirname(reactTypes)],
  baseUrl: fileURLToPath(new URL('../', import.meta.url)),
  paths: { react: [path.join(reactTypes, 'index.d.ts')], 'react/jsx-runtime': [path.join(reactTypes, 'jsx-runtime.d.ts')] }
};
const program = ts.createProgram(files, options);
const diagnostics = ts.getPreEmitDiagnostics(program);
if (diagnostics.length) {
  console.error(ts.formatDiagnosticsWithColorAndContext(diagnostics, {
    getCurrentDirectory: () => process.cwd(),
    getCanonicalFileName: (name) => name,
    getNewLine: () => '\n'
  }));
  process.exitCode = 1;
} else {
  console.log('AI resource validator and document actions type-check passed (3 source modules).');
}
