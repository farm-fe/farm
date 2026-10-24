import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';

/** JS Farm integration backed by Node's WebAssembly parser, not a JS WASM engine.
 * The same binary executes in the browser for every comparison group.
 */
export function generateWasmGlue(module, url) {
  const imports = new Map();
  for (const entry of WebAssembly.Module.imports(module)) {
    if (!imports.has(entry.module)) imports.set(entry.module, []);
    imports.get(entry.module).push(entry.name);
  }
  const namespaces = [...imports.keys()];
  const declarations = namespaces.map((name, index) => `import * as __wasmImport${index} from ${JSON.stringify(name)};`);
  const importObject = namespaces.map((name, index) => `${JSON.stringify(name)}: { ${imports.get(name).map(field => `${JSON.stringify(field)}: __wasmImport${index}[${JSON.stringify(field)}]`).join(', ')} }`).join(', ');
  const exports = WebAssembly.Module.exports(module).map(({ name }, index) => {
    const exportName = /^[$A-Z_a-z][$\w]*$/.test(name) ? name : JSON.stringify(name);
    return `const __wasmExport${index} = __wasmInstance.exports[${JSON.stringify(name)}];\nexport { __wasmExport${index} as ${exportName} };`;
  });
  return [...declarations,
    `const __wasmResponse = await fetch(${JSON.stringify(url)});`,
    `if (!__wasmResponse.ok) throw new Error('WASM request failed: ' + __wasmResponse.status);`,
    `const { instance: __wasmInstance } = await WebAssembly.instantiate(await __wasmResponse.arrayBuffer(), { ${importObject} });`,
    ...exports,
  ].join('\n');
}

/** @returns {import('@farmfe/core').JsPlugin} */
export function wasm() {
  return {
    name: 'commerce-js-wasm',
    load: {
      filters: { resolvedPaths: ['\\.wasm$'] },
      async executor(param, context) {
        const bytes = await fs.readFile(param.resolvedPath);
        // Reject malformed modules before emitting an unusable resource.
        const module = new WebAssembly.Module(bytes);
        const hash = createHash('sha256').update(bytes).digest('hex').slice(0, 16);
        const name = `assets/wasm/${path.basename(param.resolvedPath, '.wasm')}.${hash}.wasm`;
        const content = generateWasmGlue(module, `/${name}`);
        context.emitFile({ resolvedPath: param.moduleId, name, content: [...bytes], resourceType: 'asset' });
        return { content, moduleType: 'js' };
      },
    },
  };
}
