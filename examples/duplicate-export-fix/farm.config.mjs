import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      complex: './src/complex-entry.js',
      namespace: './src/namespace-entry.js'
    },
    output: {
      path: './dist',
      format: 'esm'
    },
    minify: false,
    sourcemap: false
  }
});