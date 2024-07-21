import { defineConfig } from '@farmfe/core';
import postcss from '@farmfe/js-plugin-postcss';
import * as path from "node:path";

export default defineConfig({
  plugins: ['@farmfe/plugin-react', postcss()],
  compilation: {
    output: {
      publicPath: "./"
    },
    sourcemap: false,
    persistentCache: false,
    resolve: {
      alias: {
        "@/": path.join(process.cwd(), "src"),
        "@styled-system/": path.join(process.cwd(), "styled-system"),
      },
    },
  }
});
