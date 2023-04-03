import { defineConfig } from 'tsup';
export default defineConfig({
  minify: true,
  entry: ['./index.ts'],
  platform: 'node',
  target: 'node16',
  treeshake: true,
  esbuildPlugins: [
    {
      name: 'alias',
      setup({ onResolve, resolve }) {
        onResolve(
          { filter: /^prompts$/, namespace: 'file' },
          async ({ importer, resolveDir }) => {
            const result = await resolve('prompts/lib/index.js', {
              importer,
              resolveDir,
              kind: 'import-statement',
            });
            return result;
          }
        );
      },
    },
  ],
});
