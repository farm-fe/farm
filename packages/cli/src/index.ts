import { cac } from 'cac';

import { COMMANDS } from './plugin/index.js';
import { cleanOptions, resolveCommandOptions, resolveCore } from './utils.js';
import { VERSION } from './constants.js';
import { logger } from './logger.js';

interface GlobalFarmCLIOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  m?: string;
  mode?: string;
}

interface FarmCLIServerOptions {
  port?: string;
  open?: boolean;
  https?: boolean;
  hmr?: boolean;
  strictPort?: boolean;
}

interface FarmCLIBuildOptions {
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
}

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
  //TODO add host config
  .option('--port [port]', 'specify port')
  .option('--open', 'open browser on server start')
  .option('--hmr', 'enable hot module replacement')
  // TODO add https config with core
  // .option('--https', 'use https')
  // TODO add strictPort open config with core
  // .option('--strictPort', 'specified port is already in use, exit with error')
  .action(async (options: FarmCLIServerOptions & GlobalFarmCLIOptions) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { start } = await resolveCore(resolveOptions.configPath);

      await start(cleanOptions(resolveOptions));
    } catch (e) {
      logger(e.message);
      process.exit(1);
    }
  });

// build command
cli
  .command('build', 'compile the project in production mode')
  // TODO add target config
  // .option("--target <target>", "transpile target")
  .option('--outDir <dir>', 'output directory')
  // TODO sourcemap output config path
  .option('--sourcemap', 'output source maps for build')
  .option('--minify', 'code compression at build time')
  .action(async (options: FarmCLIBuildOptions & GlobalFarmCLIOptions) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { build } = await resolveCore(resolveOptions.configPath);
      build(cleanOptions(resolveOptions));
    } catch (e) {
      logger(e.message);
      process.exit(1);
    }
  });

// watch command
// TODO add watch command
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
cli.on('command:*', () => {
  cli.outputHelp();
});

cli.help();

cli.version(VERSION);

cli.parse();
