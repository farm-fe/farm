import { copyFile } from 'node:fs/promises';

// The helper has identical public types in both formats, but NodeNext needs
// a CommonJS declaration extension for its require condition.
if (process.argv.includes('--plugin')) {
  await copyFile(
    new URL('../dist/plugin/qiankun-farm-plugin.d.ts', import.meta.url),
    new URL('../dist/plugin/qiankun-farm-plugin.d.cts', import.meta.url)
  );
} else {
  await copyFile(
    new URL('../dist/helper/qiankun-farm-plugin-helper.d.ts', import.meta.url),
    new URL('../dist/helper/qiankun-farm-plugin-helper.d.cts', import.meta.url)
  );
}
