import type { UserConfig } from '@farmfe/core';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    resolve: {
      strictExports: true
    }
  },
  server: {
    proxy: {
      '^/api': {
        target: 'https://music-erkelost.vercel.app/banner',
        changeOrigin: true,
        rewrite: (path: any) => path.replace(/^\/api/, '')
      }
    }
  }
};
