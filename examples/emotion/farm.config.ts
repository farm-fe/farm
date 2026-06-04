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
      // Match POSIX and Windows path separators because CI builds examples on all platforms.
      plugins: [
        {
          name: '@swc/plugin-emotion',
          options: {
            autoLabel: 'always',
            labelFormat: 'farm-emotion-[local]',
          },
          filters: {
            resolvedPaths: ['src[/\\\\]emotion.tsx$'],
          },
        },
        {
          name: '@swc/plugin-styled-components',
          options: {
            displayName: true,
            ssr: true,
          },
          filters: {
            resolvedPaths: ['src[/\\\\]styled-components.tsx$'],
          },
        },
        {
          name: '@swc/plugin-styled-jsx',
          options: {},
          filters: {
            resolvedPaths: ['src[/\\\\]styled-jsx.tsx$'],
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
