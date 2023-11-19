import path from 'path';

import { build } from '../dist/index.js';

build({
  configPath: path.join(process.cwd(), 'farm.config.ts')
});
