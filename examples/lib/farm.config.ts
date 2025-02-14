import { defineConfig } from '@farmfe/core';
import farmDtsPlugin from '@farmfe/js-plugin-dts';
import path from 'node:path';
export default defineConfig({
  compilation: {
    presetEnv: false,
    progress: false,
    input: {
      index: "./index.ts"
    },
    output: {
      targetEnv: 'node'
    },
    persistentCache: false,
    resolve: {
      alias: {
        "@": path.resolve("./src")
      }
    }
  },
  // plugins: [
    // farmDtsPlugin({
      // tsConfigPath: './tsconfig.json'
    // })
  // ]
  plugins: [test()]
});
function test() {
  return {
    name: "test",
    freezeModule: {
      filters: {
        // 改为具体的模块类型而不是正则 ".*"
        moduleTypes: ["ts"],
        resolvedPaths: ['.*']
      },
      async executor(param) {
        if (param.moduleId.endsWith('.ts')) {
          console.log(param.content);
          return {
            content: param.content,
          };
        }
      },
    }
  };
}
