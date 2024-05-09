import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    presetEnv: false,
    external: [
      {
        jquery: '$',
        'react-dom': 'ReactDom'
      },
      'react'
    ]
  }
});
