import path from 'path';
import { fileURLToPath } from 'url';
import { test, expect } from 'vitest';

import { resolveUserConfig } from '../src/index.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  console.log(
    '\n\n\n\n',
    filePath,
    path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
    '\n\n\n\n\n'
  );
  const config = await resolveUserConfig(
    path.join(filePath, 'fixtures', 'config', 'farm.config.ts')
  );

  expect(config).toEqual({
    compilation: {
      input: {
        main: './main.tsx',
      },
    },
  });
});
