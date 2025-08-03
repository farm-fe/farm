import { defineConfig } from "farm"

export default defineConfig({
  compilation: {
    input: {
      index: './src/button/index.ts'
    },
    output: {
      path: 'build'
    },
    partialBundling: {
      enforceResources: [
        {
          name: 'button',
          test: ['.+']
        }
      ]
    },
    external: ['^react$', '^react-dom$'],
    presetEnv: false,
    sourcemap: false,
    persistentCache: false,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
