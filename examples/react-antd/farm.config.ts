import { defineConfig } from '@farmfe/core'

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build',
      publicPath: '/admin/'
    },
    presetEnv: false,
    sourcemap: true,
    persistentCache: false,
  },
  server: {
    writeToDisk: false,
    cors: true,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});


