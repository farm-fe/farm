import { transform } from '@babel/core';
import { JsPlugin, defineConfig } from '@farmfe/core';
import postCSSPlugin from '@farmfe/js-plugin-postcss';
import stylexExtendBabelPlugin from '@stylex-extend/babel-plugin';
import stylexBabelPlugin from '@stylexjs/babel-plugin';
import Pages from 'vite-plugin-pages';
import { visualizer } from '../server';

function stylex() {
  return <JsPlugin>{
    name: 'stylex',
    transform: {
      filters: {
        moduleTypes: ['ts', 'tsx']
      },
      executor(param) {
        // console.log(param.resolvedPath);
        // console.log(param);
        if (
          param.resolvedPath === 'farmfe_plugin_react_is_react_refresh_boundary'
        ) {
          return param;
        }
        const res = transform(param.content, {
          filename: param.resolvedPath,
          parserOpts: {
            plugins: ['jsx', 'typescript']
          },
          plugins: [
            stylexExtendBabelPlugin.withOptions({
              unstable_moduleResolution: {
                type: 'commonJS',
                rootDir: __dirname
              },
              transport: 'props'
            }),
            stylexBabelPlugin.withOptions({
              dev: false,
              runtimeInjection: false,
              unstable_moduleResolution: {
                type: 'commonJS',
                rootDir: __dirname
              }
            })
          ]
        });
        if (res && 'stylex' in res.metadata) {
          if (
            Array.isArray(res.metadata.stylex) &&
            res.metadata.stylex.length > 0
          ) {
            return {
              content: res.code,
              sourceMap: res?.map,
              moduleType: 'tsx',
              ignorePreviousSourceMap: true
            };
          }
        }
        return param;
      }
    }
  };
}

export default defineConfig({
  plugins: [stylex(), '@farmfe/plugin-react', postCSSPlugin(), visualizer()],
  vitePlugins: [
    Pages({
      resolver: 'react',
      dirs: 'pages'
    })
  ]
});
