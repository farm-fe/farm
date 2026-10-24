import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

const require = createRequire(new URL('../../../js-plugins/svgr/package.json', import.meta.url));
/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, object]} */
export function commerceIcons(group, nativeFactory) {
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('commerceIcons requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ compiler: 'jsx', jsx: 'react', scale: 1, autoInstall: false,
    customCollections: { commerce: 'apps/admin/assets/icons' } });
  }
  return {
    name: 'commerce-js-local-icons',
    resolve: {
      filters: { sources: ['^~icons/commerce/(inventory|launch)$', '^virtual:commerce-icons:(inventory|launch)$'], importers: ['.*'] },
      executor(param) { return { resolvedPath: param.source.startsWith('virtual:') ? param.source : `virtual:commerce-icons:${param.source.split('/').at(-1)}`, sideEffects: false }; },
    },
    load: {
      filters: { resolvedPaths: ['^virtual:commerce-icons:(inventory|launch)$'] },
      async executor(param, context) {
        const name = param.resolvedPath.split(':').at(-1);
        const source = fileURLToPath(new URL(`../apps/admin/assets/icons/${name}.svg`, import.meta.url));
        context.addWatchFile(param.resolvedPath, source);
        const svg = await fs.readFile(source, 'utf8');
        const { transform } = await import(pathToFileURL(require.resolve('@svgr/core')).href);
        const jsx = await import(pathToFileURL(require.resolve('@svgr/plugin-jsx')).href);
        const content = await transform(svg, { jsxRuntime: 'classic', plugins: [jsx.default ?? jsx] });
        return { content, moduleType: 'jsx' };
      },
    },
  };
}
