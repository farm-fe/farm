import path from 'node:path';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';

import chalk from 'chalk';

const TEMPLATE_REACT = path.join(TEMPLATES_DIR, 'react');

export async function create(): Promise<void> {
  const dest = path.join(process.cwd(), 'farm-react');

  copyFiles(TEMPLATE_REACT, dest);

  console.log(
    chalk.green('Created a new Farm app in ') +
      chalk.bold(dest) +
      chalk.green('.')
  );
  console.log(
    `Run ${chalk.cyan.bold(
      'cd farm-react && npm i && npm start'
    )} to get started.`
  );
}
