import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    input: {
      index: 'src/index.ts',
    },
    output: {
      format: 'cjs',
      targetEnv: 'node',
      entryFilename: '[entryName].cjs',
      filename: '[name].cjs'
    }
  },
  server: {
    hmr: false,
  },
});