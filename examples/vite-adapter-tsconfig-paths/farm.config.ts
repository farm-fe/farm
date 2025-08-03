import { defineConfig } from 'farm';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  compilation: {
    input: {
      index: 'src/index.ts',
    },
    presetEnv: false,
    minify: false,
    persistentCache: false,
  },
  vitePlugins: [tsconfigPaths()],
})
