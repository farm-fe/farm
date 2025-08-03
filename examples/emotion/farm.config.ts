import type { UserConfig } from "farm";

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
