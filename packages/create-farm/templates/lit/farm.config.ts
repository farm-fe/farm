import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    script: {
      plugins: [],
      target: 'es2022',
      parser: {
        tsConfig: {
          decorators: true,
          dts: false,
          noEarlyErrors: false,
          tsx: false,
        },
      },
      decorators: {
        legacyDecorator: true,
        decoratorMetadata: false,
        decoratorVersion: '2021-12',
        includes: ["src/my-element.ts"],
        excludes: ['node_modules/'],
      }
    },
    presetEnv: false,
  },
});