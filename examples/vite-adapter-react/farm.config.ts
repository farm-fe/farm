import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';
import react from '@farmfe/plugin-react'
import { viteSingleFile } from "vite-plugin-singlefile"

export default defineConfig({
  plugins: [
    react({ runtime: 'automatic', refresh: true})
  ],
  compilation:{
    persistentCache: false,
    minify: false,
    output: {
      publicPath: '/a',
      targetEnv: 'browser',
    }
  },
  vitePlugins: [
    Pages({
      resolver: 'react'
    }),
    viteSingleFile()
  ]
});
