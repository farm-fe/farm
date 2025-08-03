import { defineConfig } from 'farm';
import Pages from 'vite-plugin-pages';
import react from '@farmfe/plugin-react'

export default defineConfig({
  plugins: [
    react({ runtime: 'automatic', refresh: true})
  ],
  compilation:{
    persistentCache: false
  },
  vitePlugins: [
    Pages({
      resolver: 'react'
    })
  ]
});
