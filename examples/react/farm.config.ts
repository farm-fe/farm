import { defineConfig } from '@farmfe/core';

export default defineConfig({
  server: {
    port: 3000
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass'
  ],
  vitePlugins: [
    {
      name: 'vite111',
      config(config, env) {
        return config;
      },
      configResolved(config) {}
    }
    // {
    //   name: 'vite2222',
    //   config(config) {
    //     return config
    //   }
    // }
  ]
});
