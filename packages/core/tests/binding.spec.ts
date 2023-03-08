import path from 'path';
import { test } from 'vitest';
import { Compiler, normalizeUserCompilationConfig } from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  const currentDir = path.dirname(new URL(import.meta.url).pathname);
  const config = await normalizeUserCompilationConfig({
    root: path.resolve(currentDir, 'fixtures', 'binding'),
  });
  const compiler = new Compiler(config);
  await compiler.compile();
});
