import fs from 'node:fs/promises';

/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, object]} */
export function productImageDom(group, nativeFactory) {
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('productImageDom requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ dom: true, include: ['product-.*\\.svg\\?dom'] });
  }
  return {
    name: 'commerce-js-product-image-dom',
    load: {
      filters: { resolvedPaths: ['product-(home|outdoor|studio|travel)\\.svg'] },
      async executor(param) {
        if (!param.query.some(([key]) => key === 'dom')) return null;
        const bytes = await fs.readFile(param.resolvedPath);
        const src = `data:image/svg+xml;base64,${bytes.toString('base64')}`;
        return { content: `const image = new Image(); image.src = ${JSON.stringify(src)}; export default image;`, moduleType: 'js' };
      },
    },
  };
}
