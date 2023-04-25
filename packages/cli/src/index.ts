import { cac } from 'cac';
import { readFileSync } from 'node:fs';
import { COMMANDS } from './plugin/index.js';
import { cleanOptions, resolveCommandOptions, resolveCore } from './utils.js';
import { createLogger } from './logger.js';
import type {
  FarmCLIBuildOptions,
  FarmCLIServerOptions,
  GlobalFarmCLIOptions
} from './types.js';

const logger = createLogger();

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

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
  // TODO add open config with core
  // .option('--open', 'open browser on server start')
  .option('--hmr', 'enable hot module replacement')
  // TODO add https config
  // .option('--https', 'use https')
  // TODO add strictPort open config with core
  // .option('--strictPort', 'specified port is already in use, exit with error')
  .action(async (options: FarmCLIServerOptions & GlobalFarmCLIOptions) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { start } = await resolveCore(resolveOptions.configPath);

      await start(cleanOptions(resolveOptions));
    } catch (e) {
      logger.error(e.message);
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
      logger.error(e.message);
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
    try {
      COMMANDS[command](args);
    } catch (e) {
      logger.error(
        'The command arg parameter is incorrect. Please check whether you entered the correct parameter. such as "farm create plugin"'
      );
      process.exit(1);
    }
  });

// Listening for unknown command
cli.on('command:*', () => {
  logger.error(
    'Unknown command place Run "my-cli --help" to see available commands'
  );
});

cli.help();

cli.version(version);

cli.parse();
