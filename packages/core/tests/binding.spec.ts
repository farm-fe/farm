// import path from 'path';
// import { fileURLToPath } from 'url';
import { test } from 'vitest';
// import { Compiler } from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  // const compiler = new Compiler({
  //   config: {
  //     input: { index: './index.html' },
  //     root: path.join(__dirname, 'fixtures', 'binding'),
  //     runtime: {
  //       path: path.dirname(require.resolve('@farmfe/runtime/package.json')),
  //     },
  //     resolve: {
  //       alias: {
  //         '@swc/helpers': path.dirname(
  //           require.resolve('@swc/helpers/package.json')
  //         ),
  //       },
  //       symlinks: true,
  //     },
  //   },
  //   jsPlugins: [],
  //   rustPlugins: [],
  // });
  // await compiler.compile();
});
