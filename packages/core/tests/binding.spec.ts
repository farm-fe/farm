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
  const config = await normalizeUserCompilationConfig(
    null,
    {
      root: path.resolve(currentDir, 'fixtures', 'binding')
    },
    new DefaultLogger()
  );
  const compiler = new Compiler(config);
  await compiler.compile();
});
