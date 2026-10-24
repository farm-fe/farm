import { gzip } from 'node:zlib';
import { promisify } from 'node:util';

const gzipAsync = promisify(gzip);

/** Farm integration using Node's maintained zlib engine, not a JS reimplementation.
 * @returns {import('@farmfe/core').JsPlugin}
 */
export function compression() {
  return {
    name: 'commerce-js-gzip',
    finalizeResources: {
      async executor({ resourcesMap }) {
        const result = { ...resourcesMap };
        await Promise.all(Object.entries(resourcesMap).map(async ([name, resource]) => {
          if (resource.emitted || resource.bytes.length < 1024 || !/\.(js|mjs|json|css|html)$/.test(name)) return;
          const compressed = await gzipAsync(Buffer.from(resource.bytes), { level: 6 });
          result[`${name}.gz`] = {
            ...resource, name: `${name}.gz`, bytes: [...compressed], emitted: false, resourceType: 'asset',
          };
        }));
        return result;
      },
    },
  };
}
