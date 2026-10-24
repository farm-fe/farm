import rollupStrip from '@rollup/plugin-strip';
import { parse } from 'acorn';

/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {false | import('@farmfe/core').JsPlugin | [string, object]} */
export function publicationStrip(group, mode = process.env.NODE_ENV ?? 'development', nativeFactory) {
  if (!['development', 'production'].includes(mode)) throw new Error('Unsupported strip mode');
  if (mode === 'development') return false;
  const include = ['publication-diagnostics\\.mjs$'];
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('publicationStrip requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ include, functions: ['commerceDiagnostics\\.trace'], debugger: false, sourceMap: true });
  }
  const transform = rollupStrip({ include: /.*/, functions: ['commerceDiagnostics.trace'], debugger: false, sourceMap: true }).transform;
  return {
    name: 'commerce-js-publication-strip',
    transform: {
      filters: { resolvedPaths: include },
      executor(param) {
        const result = transform.call({
          parse(code) { return parse(code, { ecmaVersion: 'latest', sourceType: 'module' }); },
          error(error) { throw new Error(String(error)); },
        }, param.content, param.resolvedPath);
        if (!result) return null;
        return { content: result.code, moduleType: param.moduleType, sourceMap: result.map?.toString() };
      },
    },
  };
}
