import fs from 'fs';
import path from 'path';

import { EXAMPLES_DIR, JS_PLUGINs_DIR } from './build.mjs';

[JS_PLUGINs_DIR, EXAMPLES_DIR].forEach((dir) => {
  if (fs.existsSync(dir)) {
    console.log(`Clearing cache under ${dir}`);
    const files = fs.readdirSync(dir);
    files.forEach((file) => {
      const filePath = path.join(dir, file, 'node_modules', '.farm');

      if (fs.existsSync(filePath) && fs.lstatSync(filePath).isDirectory()) {
        fs.rmSync(filePath, { recursive: true, force: true });
      }
    });
  }
});

console.log('Cache cleared');
