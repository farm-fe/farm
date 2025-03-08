import { defineConfig } from "@farmfe/core";
import less from '@farmfe/js-plugin-less';
import postcss from "@farmfe/js-plugin-postcss";

const isDevelopment = process.env.NODE_ENV === 'development';

export default defineConfig({
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
    minify: false,
    persistentCache: false,
    presetEnv: {
      include: isDevelopment ? [] : ['node_modules/*'],
      options: {
        targets: 'Chrome >= 49',
      },
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
    less({/* options */}),
    postcss({/* options */}),
    // svgr({/* options */}),
  ],
});