import { test } from 'vitest';
import { Compiler } from '../src/index';

test('Binding - should parse config to rust correctly', async () => {
  const compiler = new Compiler({
    input: { index: './index.html' },
    plugins: [],
  });

  await compiler.compile();
});
