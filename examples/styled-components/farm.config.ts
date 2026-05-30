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
          name: '@swc/plugin-styled-components',
          options: {
            displayName: true,
            ssr: true,
          },
          filters: {
            moduleTypes: ['tsx'],
          },
        },
      ],
    },
    sourcemap: false,
  },
  plugins: ['@farmfe/plugin-react'],
});
