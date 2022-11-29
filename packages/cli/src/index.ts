import path from 'path';

import cac from 'cac';

import { start } from './start';
import { COMMANDS } from './plugin';

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

const pluginCmd = cli.command(
  'plugin <command>',
  'Commands for manage plugins',
  {
    allowUnknownOptions: true,
  }
);
pluginCmd.action((command: 'build' | 'create', args: any) => {
  COMMANDS[command](args);
});

pluginCmd.cli.help();

cli.help();

try {
  cli.parse();
} catch (e) {
  // TODO error handling
  console.log(e);
}
