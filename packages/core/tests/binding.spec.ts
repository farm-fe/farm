import path from 'path';
import { fileURLToPath } from 'url';
import { test } from 'vitest';
import { Compiler, normalizeUserCompilationConfig } from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  const config = await normalizeUserCompilationConfig({
    root: path.resolve(currentDir, 'fixtures', 'binding')
  });
  const compiler = new Compiler(config);
  await compiler.compile();
});
