import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index-server.tsx'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format:"cjs"
    }
  },
  plugins: [
    [
      '@farmfe/plugin-react',
      {
        refresh: false,
        development: false
      }
    ]
  ]
});
