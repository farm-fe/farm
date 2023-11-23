import { buildCli, buildCoreCjs, buildJsPlugins } from './build.mjs';

import fs from 'fs';
import path from 'path';
import { execa } from 'execa';

console.log('Building CLI...');
await buildCli();
console.log('Building core CJS...');
await buildCoreCjs();
console.log('Building JS plugins...');
await buildJsPlugins();

// read all directories under examples and run `npm run build` in each directory
const examples = fs.readdirSync('./examples');
console.log('Building', examples.length, 'examples...');

for (const example of examples) {
  const examplePath = path.join('./examples', example);
  console.log('Building', examplePath);

  if (fs.statSync(examplePath).isDirectory()) {
    await execa('npm', ['run', 'build'], {
      cwd: examplePath
    });
  }
}
