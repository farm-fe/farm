import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.resolve('@babel/plugin-transform-react-jsx'));
const babel = require('@babel/core');
const isJsx = node => node?.type === 'JSXElement' || node?.type === 'JSXFragment';
function returnsJsx(node) {
  if (!node) return false;
  if (node.type === 'FunctionDeclaration' || node.type === 'ArrowFunctionExpression' || node.type === 'FunctionExpression') {
    return isJsx(node.body) || node.body?.body?.some(statement => statement.type === 'ReturnStatement' && isJsx(statement.argument));
  }
  return false;
}

async function discover(directory) {
  const result = new Map();
  async function walk(current) {
    for (const entry of (await fs.readdir(current, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const filename = path.join(current, entry.name);
      if (entry.isDirectory()) { await walk(filename); continue; }
      if (!/\.[jt]sx$/.test(filename)) continue;
      const ast = await babel.parseAsync(await fs.readFile(filename, 'utf8'), {
        filename, configFile: false, babelrc: false, parserOpts: { plugins: ['typescript', 'jsx'] },
      });
      for (const statement of ast.program.body) {
        if (statement.type !== 'ExportNamedDeclaration') continue;
        const declaration = statement.declaration;
        const exports = declaration?.type === 'VariableDeclaration'
          ? declaration.declarations.filter(item => returnsJsx(item.init)).map(item => item.id.name)
          : returnsJsx(declaration) ? [declaration.id.name] : [];
        for (const name of exports) {
          if (!/^[A-Z]/.test(name)) continue;
          const matches = result.get(name) ?? [];
          matches.push(filename);
          result.set(name, matches);
        }
      }
    }
  }
  await walk(directory);
  return result;
}

/** Named local JSX exports via mature Babel parsing/traversal and a thin discovery policy.
 * Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, object]}
 */
export function reactComponents(group, nativeFactory) {
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('reactComponents requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ dirs: ['apps/admin/components'], local: true,
    importMode: 'relative', dts: false, include: ['ProductImagePreview\\.tsx$'] });
  }
  const catalog = discover(path.resolve('apps/admin/components'));
  return {
    name: 'commerce-js-local-components',
    transform: {
      filters: { resolvedPaths: ['ProductImagePreview\\.tsx$'] },
      async executor(param) {
        const components = await catalog;
        const result = await babel.transformAsync(param.content, {
          filename: param.resolvedPath, configFile: false, babelrc: false, sourceMaps: true,
          parserOpts: { plugins: ['typescript', 'jsx'] },
          plugins: [({ types }) => ({ visitor: { Program(program) {
            const selected = new Map();
            program.traverse({ JSXOpeningElement(element) {
              const name = element.node.name.type === 'JSXIdentifier' ? element.node.name.name : null;
              if (!name || element.scope.hasBinding(name) || !components.has(name)) return;
              const matches = components.get(name);
              if (matches.length !== 1) throw new Error(`Ambiguous local component: ${name}`);
              selected.set(name, matches[0]);
            } });
            for (const [name, filename] of [...selected].sort(([a], [b]) => a.localeCompare(b))) {
              const relative = './' + path.relative(path.dirname(param.resolvedPath), filename).replaceAll('\\', '/');
              program.unshiftContainer('body', types.importDeclaration([
                types.importSpecifier(types.identifier(name), types.identifier(name)),
              ], types.stringLiteral(relative)));
            }
          } } })],
        });
        return { content: result.code, moduleType: param.moduleType, sourceMap: JSON.stringify(result.map) };
      },
    },
  };
}
