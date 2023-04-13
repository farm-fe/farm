import { createHash } from 'node:crypto';
import type { types } from '@babel/core';
import * as babel from '@babel/core';
import jsx from '@vue/babel-plugin-jsx';
import { JsPlugin } from '@farmfe/core';
// eslint-disable-next-line node/no-extraneous-import
import type { CallExpression, Identifier } from '@babel/types';
import type { Options } from './types';
import { resolveExcludes, resolveIncludes } from './utils';

export * from './types';

function vueJsxPlugin(options: Options = {}): JsPlugin {
  let root = '';
  let needHmr = false;
  let needSourceMap = true;

  const {
    babelPlugins = [],
    include,
    exclude,
    ...babelPluginOptions
  } = options;
  const resolvedIncludes = include ? resolveIncludes(include) : [];
  const resolvedExcludes = exclude ? resolveExcludes(exclude) : [];

  return {
    name: 'farm-vue-jsx-plugin',
    config(config) {
      config.define = {
        __VUE_OPTIONS_API__: config.define?.__VUE_OPTIONS_API__ ?? 'true',
        __VUE_PROD_DEVTOOLS__: config.define?.__VUE_PROD_DEVTOOLS__ ?? 'false',
      };

      //should hmr or sourcemap depends on "mode"
      needHmr = config.mode === 'development';
      needSourceMap = config.mode === 'development';
      // root = config.input;
      return config;
    },
    transform: {
      filters: {
        resolvedPaths: ['.jsx$', '.tsx$', ...resolvedIncludes],
      },
      async executor({ content, resolvedPath, query, meta }) {
        const defaultReturn = { content: '', moduleType: 'js', sourceMap: '' };
        for (let i = 0; i < resolvedExcludes.length; i++) {
          if (resolvedExcludes[i].test(resolvedPath)) return defaultReturn;
        }

        //default babel plugins
        const plugins = [[jsx, babelPluginOptions], ...babelPlugins];

        //vue-jsx-plugin should compatible ".jsx" and ".tsx"
        //so there needs to handle ".tsx" file.
        if (resolvedPath.endsWith('.tsx')) {
          plugins.push([
            // @ts-ignore missing type
            await import('@babel/plugin-transform-typescript').then(
              (r) => r.default
            ),
            { isTSX: true, allowExtensions: true },
          ]);
        }

        //production
        if (!needHmr) {
          plugins.push(() => {
            return {
              visitor: {
                CallExpression: {
                  enter(_path: babel.NodePath<CallExpression>) {
                    if (isDefineComponentCall(_path.node)) {
                      const callee = _path.node.callee as Identifier;
                      callee.name = `/* @__PURE__ */ ${callee.name}`;
                    }
                  },
                },
              },
            };
          });
        }

        //transform "jsx" or "tsx" to "js"
        const result = babel.transformSync(content, {
          babelrc: false,
          ast: true,
          plugins,
          sourceMaps: needSourceMap,
          sourceFileName: resolvedPath,
          configFile: false,
        })!;

        if (!needHmr) {
          if (!result.code) return defaultReturn;
          return {
            content: result.code,
            sourceMap: JSON.stringify(result.map),
            moduleType: 'js',
          };
        }

        interface HotComponent {
          local: string;
          exported: string;
          id: string;
        }

        // check for hmr injection
        const declaredComponents: { name: string }[] = [];
        const hotComponents: HotComponent[] = [];
        let hasDefault = false;

        for (const node of result.ast!.program.body) {
          if (node.type === 'VariableDeclaration') {
            const names = parseComponentDecls(node, content);
            if (names.length) {
              declaredComponents.push(...names);
            }
          }

          if (node.type === 'ExportNamedDeclaration') {
            if (
              node.declaration &&
              node.declaration.type === 'VariableDeclaration'
            ) {
              hotComponents.push(
                ...parseComponentDecls(node.declaration, content).map(
                  ({ name }) => ({
                    local: name,
                    exported: name,
                    id: getHash(resolvedPath + name),
                  })
                )
              );
            } else if (node.specifiers.length) {
              for (const spec of node.specifiers) {
                if (
                  spec.type === 'ExportSpecifier' &&
                  spec.exported.type === 'Identifier'
                ) {
                  const matched = declaredComponents.find(
                    ({ name }) => name === spec.local.name
                  );
                  if (matched) {
                    hotComponents.push({
                      local: spec.local.name,
                      exported: spec.exported.name,
                      id: getHash(resolvedPath + spec.exported.name),
                    });
                  }
                }
              }
            }
          }

          if (node.type === 'ExportDefaultDeclaration') {
            if (node.declaration.type === 'Identifier') {
              const _name = node.declaration.name;
              const matched = declaredComponents.find(
                ({ name }) => name === _name
              );
              if (matched) {
                hotComponents.push({
                  local: node.declaration.name,
                  exported: 'default',
                  id: getHash(resolvedPath + 'default'),
                });
              }
            } else if (isDefineComponentCall(node.declaration)) {
              hasDefault = true;
              hotComponents.push({
                local: '__default__',
                exported: 'default',
                id: getHash(resolvedPath + 'default'),
              });
            }
          }
        }

        if (hotComponents.length) {
          if (hasDefault && needHmr) {
            result.code =
              result.code!.replace(
                /export default defineComponent/g,
                `const __default__ = defineComponent`
              ) + `\nexport default __default__`;
          }

          if (needHmr) {
            let code = result.code;
            let callbackCode = ``;
            for (const { local, exported, id } of hotComponents) {
              code +=
                `\n${local}.__hmrId = "${id}"` +
                `\n__VUE_HMR_RUNTIME__.createRecord("${id}", ${local})`;
              callbackCode += `\n__VUE_HMR_RUNTIME__.reload("${id}", __${exported})`;
            }

            code += `\nmodule.meta.hot.accept(({${hotComponents
              .map((c) => `${c.exported}: __${c.exported}`)
              .join(',')}}) => {${callbackCode}\n})`;

            result.code = code;
          }
        }

        if (!result.code) return defaultReturn;
        return {
          content: result.code,
          sourceMap: JSON.stringify(result.map),
          moduleType: 'js',
        };
      },
    },
  };
}

function parseComponentDecls(node: types.VariableDeclaration, source: string) {
  const names = [];
  for (const decl of node.declarations) {
    if (decl.id.type === 'Identifier' && isDefineComponentCall(decl.init)) {
      names.push({
        name: decl.id.name,
      });
    }
  }
  return names;
}

function isDefineComponentCall(node?: types.Node | null) {
  return (
    node &&
    node.type === 'CallExpression' &&
    node.callee.type === 'Identifier' &&
    node.callee.name === 'defineComponent'
  );
}

function getHash(text: string) {
  return createHash('sha256').update(text).digest('hex').substring(0, 8);
}

export default vueJsxPlugin;
