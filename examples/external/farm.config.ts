import { defineConfig } from "farm";

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
