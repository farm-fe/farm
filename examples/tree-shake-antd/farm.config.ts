import { defineConfig } from 'farm'

export default defineConfig({
  compilation: {
    presetEnv: false,
    sourcemap: false,
    persistentCache: false,
    minify: false
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
