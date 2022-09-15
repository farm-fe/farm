import path from 'path';
import { test } from 'vitest';
import { Compiler } from '../src/index';

test('Binding - should parse config to rust correctly', async () => {
  const compiler = new Compiler({
    compilation: {
      input: { index: './index.html' },
      root: path.join(__dirname, 'fixtures', 'binding'),
    },
    plugins: [],
  });

  await compiler.compile();
});
