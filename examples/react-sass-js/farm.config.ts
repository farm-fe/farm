import type { UserConfig } from 'farm';
import farmSassPlugin from '@farmfe/js-plugin-sass';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    resolve: {
      symlinks: true,
    },
    output: {
      path: './build',
    },
    sourcemap: false
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
  },
  plugins: [
    '@farmfe/plugin-react', 
    farmSassPlugin({
      // globals: ['./src/variables.scss']
    })
  ],
});
