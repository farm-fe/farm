import { cac } from 'cac';
import { readFileSync } from 'node:fs';
import { COMMANDS } from './plugin/index.js';
import { resolveCommandOptions, resolveCore } from './utils.js';
import { createLogger } from './logger.js';
import type {
  FarmCLIBuildOptions,
  FarmCLIPreviewOptions,
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
    '[root]',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .alias('start')
  .option('--host [host]', 'specify host')
  .option('--port <port>', 'specify port')
  .option('--open', 'open browser on server start')
  .option('--hmr', 'enable hot module replacement')
  .option('--https', 'use https')
  .option('--strictPort', 'specified port is already in use, exit with error')
  .action(async (options: FarmCLIServerOptions & GlobalFarmCLIOptions) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { start } = await resolveCore(resolveOptions.configPath);
      // TODO config with merge level
      await start(resolveOptions);
    } catch (e) {
      logger.error(`Failed to start server:\n ${e.stack}`);
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
      build(resolveOptions);
    } catch (e) {
      logger.error(`error during build:\n${e.stack}`);
      process.exit(1);
    }
  });

cli
  .command('watch', 'watch file change')
  .action(async (options: FarmCLIBuildOptions & GlobalFarmCLIOptions) => {
    try {
      const cwd = process.cwd();
      const resolveOptions = resolveCommandOptions(options);
      resolveOptions.watchPath = cwd;
      const { watch } = await resolveCore(resolveOptions.configPath);
      watch(resolveOptions);
    } catch (e) {
      logger.error(`error during watch project:\n${e.stack}`);
      process.exit(1);
    }
  });

cli
  .command('preview', 'compile the project in watch mode')
  .option('--port [port]', 'specify port')
  .option('--open', 'open browser on server preview start')
  .action(async (options: FarmCLIPreviewOptions & GlobalFarmCLIOptions) => {
    const resolveOptions = resolveCommandOptions(options);
    try {
      const { preview } = await resolveCore(resolveOptions.configPath);
      preview(resolveOptions);
    } catch (e) {
      logger.error(`Failed to start server:\n${e.stack}`);
      process.exit(1);
    }
  });

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
        `The command arg parameter is incorrect. If you want to create a plugin in farm. such as "farm create plugin"\n${e.stack}`
      );
      process.exit(1);
    }
  });

// Listening for unknown command
cli.on('command:*', () => {
  logger.error(
    'Unknown command place Run "farm --help" to see available commands'
  );
});

cli.help();

cli.version(version);

cli.parse();
