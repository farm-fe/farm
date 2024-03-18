import path from 'path';
// import { copyFile, readdir } from 'fs/promises';

import { build } from '../dist/index.js';

await build({
  configPath: path.join(process.cwd(), 'farm.config.ts')
});

// if (!process.env.FARM_PUBLISH) {
//   // copy artifacts
//   const files = await readdir(path.join(process.cwd(), 'binding'));
//   const nodeFiles = files.filter((file) => file.endsWith('.node'));

//   for (const file of nodeFiles) {
//     await copyFile(
//       path.join(process.cwd(), 'binding', file),
//       path.join(process.cwd(), 'cjs', file)
//     );
//   }
// }
