import { cac, Command } from 'cac';
import chalk from 'chalk';

import { COMMANDS } from './plugin/index.js';
import { cleanOptions, resolveCommandOptions, resolveCore } from './utils.js';
import { VERSION } from './constants.js';

const cli = cac('farm');

// common command
cli
  .option('-c, --config <file>', `use specified config file`)
  .option('-m, --mode <mode>', `set env mode`);

// dev command
cli
  .command(
    '',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .alias('start')
  .alias('dev')
  //TODO add host config
  .option('--port [port]', 'specify port')
  .option('--open', 'open browser on server start')
  // TODO add https config with core
  .option('--https', 'use https')
  // TODO add strictPort open config with core
  .option('--hmr', 'enable hot module replacement')
  .option('--strictPort', 'specified port is already in use, exit with error')
  .action(async (options) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { start } = await resolveCore(resolveOptions.configPath);

      await start(cleanOptions(resolveOptions));
    } catch (e) {
      // TODO refactor logger
      process.exit(1);
    }
  });

// build command
cli
  .command('build', 'compile the project in production mode')
  // TODO add target config
  .option('--outDir <dir>', 'output directory')
  // TODO sourcemap output config path
  .option('--sourcemap', 'output source maps for build')
  .option('--minify', 'code compression at build time')
  .action(async (options: any) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { build } = await resolveCore(resolveOptions.configPath);
      build(cleanOptions(resolveOptions));
    } catch (e) {
      process.exit(1);
    }
  });

// TODO add watch command
// watch command
cli.command('watch', 'rebuilds when files have changed on disk');

// create plugins command
cli
  .command('plugin [command]', 'Commands for manage plugins', {
    allowUnknownOptions: true
  })
  // TODO refactor plugin command
  .action((command: keyof typeof COMMANDS, args: unknown) => {
    COMMANDS[command](args);
  });

// Listening for unknown command
cli.on('command:*', function (obj: { args: string[] }) {
  const availableCommands = cli.commands.map((cmd: Command) => cmd.name);
  console.log(chalk.red(`Unknown command：${obj.args[0]}`));
  if (availableCommands.length > 0) {
    console.log(
      chalk.red(`Available commands：${availableCommands.join(',')}`)
    );
  }
});

cli.help();

cli.version(VERSION);

cli.parse();
