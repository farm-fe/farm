import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build',
      publicPath: '/admin/'
    },
    sourcemap: true,
    persistentCache: true
  },
  server: {
    writeToDisk: false,
    cors: true,
    proxy: {
      '/api': {
        target: 'https://music-erkelost.vercel.app/banner',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      }
    }
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
