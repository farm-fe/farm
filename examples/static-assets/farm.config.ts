import { defineConfig } from '@farmfe/core';
import sass from '@farmfe/js-plugin-sass';

export default defineConfig({
  plugins: [
    sass({
      legacy: true
    })
  ]
})