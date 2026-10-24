import fs from 'node:fs/promises';
import path from 'node:path';

/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, object]} */
export function productUrl(group, nativeFactory) {
  const include = ['product-(home|outdoor|studio|travel)\\.svg'];
  const limit = 550;
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('productUrl requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ include, exclude: ['\\?dom'], limit, emitFiles: true });
  }
  return {
    name: 'commerce-js-product-url',
    load: {
      filters: { resolvedPaths: include },
      async executor(param, context) {
        if (param.query.some(([key]) => key === 'dom')) return null;
        const bytes = await fs.readFile(param.resolvedPath);
        let url;
        if (bytes.length <= limit) {
          url = `data:image/svg+xml;base64,${bytes.toString('base64')}`;
        } else {
          const name = path.basename(param.resolvedPath);
          context.emitFile({ resolvedPath: param.moduleId, name, content: [...bytes], resourceType: 'asset' });
          url = `/${name}`;
        }
        return { content: `export default ${JSON.stringify(url)};`, moduleType: 'js' };
      },
    },
  };
}
