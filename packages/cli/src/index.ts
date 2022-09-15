import path from 'path';

import cac from 'cac';

import { start } from './start';

const cli = cac();

cli
  .command(
    'start',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .action((...args) => {
    console.log(args);
    start();
  });

cli.parse();
