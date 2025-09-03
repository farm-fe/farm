import { defineConfig } from "farm";
import less from '@farmfe/js-plugin-less';
import postcss from "@farmfe/js-plugin-postcss";

export default defineConfig(env => {
  const isDevelopment = env.mode === 'development';
  return {
  compilation: {
    sourcemap: isDevelopment ? 'all' : false,
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
      publicPath: "/",
      targetEnv: 'browser-legacy',
    },
    script: {
      target: "es5",
    },
    persistentCache: false,
    presetEnv: {
      include: isDevelopment ? [] : ['node_modules/*'],
    },
    css: {
      prefixer: {
       targets: ['ie >= 10']
      }
    },
  },
  server: {
    port: 1234,
    open: true,
  },
  plugins: [
    "@farmfe/plugin-react",
    "@farmfe/plugin-sass",
    {
      name: 'update-modules',
      updateModules: { executor: async (param) => {
        console.log(param)
      }}
    },
    less({/* options */}),
    postcss({/* options */}),
    // svgr({/* options */}),
  ],
}});
