import { defineConfig } from '@farmfe/core'

export default defineConfig({
  compilation: {
    presetEnv: false,
    sourcemap: false,
    persistentCache: false,
    concatenateModules: false,
    minify: false
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
