import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'browser-esnext',
    },
    persistentCache: false,
    minify: false
  },
  plugins: ['@farmfe/plugin-react']
});
