import { cac } from 'cac';
import { create } from './create/index.js';
import { COMMANDS } from './plugin/index.js';
import { resolveCore } from './utils.js';

const cli = cac();

cli
  .command(
    'start',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .action(async () => {
    const cwd = process.cwd();
    const { start } = await resolveCore(cwd);
    // TODO set config path
    start({
      configPath: cwd,
    });
  });

cli
  .command('build', 'Compile the project in production mode')
  .action(async () => {
    const cwd = process.cwd();
    const { build } = await resolveCore(cwd);
    // TODO set config path
    build({
      configPath: cwd,
    });
  });

cli.command('create [name]', 'Create a new project').action((name: string) => {
  create(name);
});

cli.command('').action(() => {
  cli.outputHelp();
});

const pluginCmd = cli.command(
  'plugin <command>',
  'Commands for manage plugins',
  {
    allowUnknownOptions: true,
  }
);
pluginCmd.action((command: keyof typeof COMMANDS, args: any) => {
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
