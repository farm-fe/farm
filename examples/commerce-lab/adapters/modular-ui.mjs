import { createRequire } from 'node:module';
import babelImport from 'babel-plugin-import';
import jsxTransform from '@babel/plugin-transform-react-jsx';

const require = createRequire(import.meta.resolve('@babel/plugin-transform-react-jsx'));
const babel = require('@babel/core');
/** JS-reference JSX lowering using the existing Babel dependency.
 * babel-plugin-import requires component references in call expressions.
 * @returns {import('@farmfe/core').JsPlugin}
 */
export function modularUiJsxPrelude({ catalog = false } = {}) {
  return {
    name: 'commerce-shared-modular-jsx', priority: 110,
    transform: {
      filters: { resolvedPaths: ['ProductBulkActions\\.tsx$', ...(catalog ? ['packages[\\\\/]extensions[\\\\/](catalog-\\d+|business-(smoke|\\d+))[\\\\/].*\\.[jt]sx$'] : [])] },
      async executor(param) {
        const result = await babel.transformAsync(param.content, {
          filename: param.resolvedPath, babelrc: false, configFile: false, sourceMaps: true,
          parserOpts: { plugins: ['typescript', 'jsx'] },
          plugins: [[jsxTransform.default ?? jsxTransform, { runtime: 'automatic' }]],
        });
        return { content: result.code, moduleType: param.moduleType, sourceMap: JSON.stringify(result.map) };
      },
    },
  };
}
/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, object]} */
export function modularUi(group, nativeFactory) {
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('modularUi requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ libraryName: '@arco-design/web-react', libDir: 'es',
    camel2Dash: false, styleLibDir: 'es', styleLibraryPath: 'style/index.css' });
  }
  return {
    name: 'commerce-js-modular-ui',
    transform: {
      filters: { resolvedPaths: ['ProductBulkActions\\.tsx$'] },
      async executor(param) {
        const result = await babel.transformAsync(param.content, {
          filename: param.resolvedPath, babelrc: false, configFile: false, sourceMaps: true,
          parserOpts: { plugins: ['typescript', 'jsx'] },
          plugins: [[babelImport.default ?? babelImport, { libraryName: '@arco-design/web-react',
            libraryDirectory: 'es', camel2DashComponentName: false,
            style: component => `${component}/style/index.css` }]],
        });
        return { content: result.code, moduleType: param.moduleType, sourceMap: JSON.stringify(result.map) };
      },
    },
  };
}
