import { defineConfig } from "@farmfe/core";
import dts from "@farmfe/js-plugin-dts";

export default defineConfig({
  compilation: {
    external: ["@farmfe/core", '@babel/core'],
    input: {
      index: "./src/index.ts",
    },
    output: {
      targetEnv: "library-node",
      path: `./dist`,
      format: "cjs",
    },
    partialBundling: {
      enforceResources: [
        {
          name: "index",
          test: [".+"],
        },
      ],
    },
    minify: false,
    sourcemap: true,
    resolve: {
      autoExternalFailedResolve: true,
    },
  },
  plugins: [dts()],
});
