import { start, build } from '@farmfe/core';
import cac from 'cac';
import { COMMANDS } from './plugin/index';

const cli = cac();

cli
  .command(
    'start',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .action((...args) => {
    console.log(args);
    // TODO set config path
    start({
      configPath: process.cwd(),
    });
  });

cli
  .command('build', 'Compile the project in production mode')
  .action((...args) => {
    console.log(args);
    // TODO set config path
    build({
      configPath: process.cwd(),
    });
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
