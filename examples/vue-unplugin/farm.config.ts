import { defineConfig } from '@farmfe/core'
import Vue from 'unplugin-vue/farm'

export default defineConfig({
  plugins: [Vue()],
  compilation: {
    persistentCache: false,
  },
})
