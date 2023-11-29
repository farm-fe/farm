import path from 'path';
import { fileURLToPath } from 'url';
import { test } from 'vitest';
import {
  Compiler,
  DefaultLogger,
  normalizeUserCompilationConfig
} from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  const compilationConfig = await normalizeUserCompilationConfig(
    null,
    {
      root: path.resolve(currentDir, 'fixtures', 'binding')
    },
    new DefaultLogger()
  );
  const compiler = new Compiler(compilationConfig);
  await compiler.compile();
});
