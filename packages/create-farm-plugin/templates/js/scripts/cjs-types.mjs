import { access, writeFile } from 'node:fs/promises';

// Preserve the generated ESM signature while declaring the callable CJS export.
await access(new URL('../build/esm/index.d.ts', import.meta.url));
await writeFile(
  new URL('../build/cjs/index.d.cts', import.meta.url),
  `declare const plugin: typeof import('../esm/index.js', { with: { 'resolution-mode': 'import' } }).default;\nexport = plugin;\n`
);
