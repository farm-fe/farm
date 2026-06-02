import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    script: {
      plugins: [
        {
          name: '@swc/plugin-emotion',
          options: {
            autoLabel: 'always',
            labelFormat: 'farm-emotion-[local]',
          },
          filters: {
            moduleTypes: ['tsx'],
          },
        },
        {
          name: '@swc/plugin-styled-components',
          options: {
            displayName: true,
            ssr: true,
          },
          filters: {
            moduleTypes: ['tsx'],
          },
        },
        {
          name: '@swc/plugin-styled-jsx',
          options: {},
          filters: {
            moduleTypes: ['tsx'],
          },
        },
      ],
    },
    sourcemap: false
    // treeShaking: true,
    // minify: true,
  },
  plugins: [['@farmfe/plugin-react', { importSource: '@emotion/react' }]],
});
