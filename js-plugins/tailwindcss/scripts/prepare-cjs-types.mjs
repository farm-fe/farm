import {copyFile} from 'node:fs/promises';

await copyFile(
  new URL('../types/index.d.cts', import.meta.url),
  new URL('../dist/index.d.cts', import.meta.url)
);
