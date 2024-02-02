import { readFileSync } from 'node:fs';
import path from 'node:path';

import { cac } from 'cac';
import {
  resolveCore,
  getConfigPath,
  resolveCommandOptions,
  handleAsyncOperationErrors,
  preventExperimentalWarning
} from './utils.js';
import { COMMANDS } from './plugin/index.js';

import type {
  FarmCLIBuildOptions,
  FarmCLIPreviewOptions,
  FarmCLIServerOptions,
  GlobalFarmCLIOptions,
  ICleanOptions
} from './types.js';

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

const cli = cac('farm');

// common command
cli
  .option('-c, --config <file>', 'use specified config file')
  .option('-m, --mode <mode>', 'set env mode')
  .option('--base <path>', 'public base path')
  .option('--clearScreen', 'allow/disable clear screen when logging', {
    default: true
  });

// dev command
cli
  .command(
    '[root]',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .alias('start')
  .option('-l, --lazy', 'lazyCompilation')
  .option('--host <host>', 'specify host')
  .option('--port <port>', 'specify port')
  .option('--open', 'open browser on server start')
  .option('--hmr', 'enable hot module replacement')
  .option('--cors', 'enable cors')
  .option('--strictPort', 'specified port is already in use, exit with error')
  .action(
    async (
      root: string,
      options: FarmCLIServerOptions & GlobalFarmCLIOptions
    ) => {
      const resolveOptions = resolveCommandOptions(options);
      const configPath = getConfigPath(options.config);

      if (root && !path.isAbsolute(root)) {
        root = path.resolve(process.cwd(), root);
      }

      const defaultOptions = {
        root,
        compilation: {
          lazyCompilation: options.lazy
        },
        server: resolveOptions,
        clearScreen: options.clearScreen,
        configPath,
        mode: options.mode
      };

      const { start } = await resolveCore();
      handleAsyncOperationErrors(
        start(defaultOptions),
        'Failed to start server'
      );
    }
  );

// build command
cli
  .command('build', 'compile the project in production mode')
  .option('-o, --outDir <dir>', 'output directory')
  .option('-i, --input <file>', 'input file path')
  .option('-w, --watch', 'watch file change')
  .option('--targetEnv <target>', 'transpile targetEnv node, browser')
  .option('--format <format>', 'transpile format esm, commonjs')
  .option('--sourcemap', 'output source maps for build')
  .option('--treeShaking', 'Eliminate useless code without side effects')
  .option('--minify', 'code compression at build time')
  .action(async (options: FarmCLIBuildOptions & GlobalFarmCLIOptions) => {
    const configPath = getConfigPath(options.config);
    const defaultOptions = {
      compilation: {
        watch: options.watch,
        output: {
          targetEnv: options?.targetEnv,
          format: options?.format
        },
        input: {
          index: options?.input
        },
        sourcemap: options.sourcemap,
        minify: options.minify,
        treeShaking: options.treeShaking
      },
      mode: options.mode,
      configPath
    };

    const { build } = await resolveCore();
    handleAsyncOperationErrors(build(defaultOptions), 'error during build');
  });

cli
  .command('watch', 'watch file change')
  .option('--format <format>', 'transpile format esm, commonjs')
  .option('-o, --outDir <dir>', 'output directory')
  .option('-i, --input <file>', 'input file path')
  .action(async (options: FarmCLIBuildOptions & GlobalFarmCLIOptions) => {
    const configPath = getConfigPath(options.config);
    const defaultOptions = {
      mode: options.mode,
      compilation: {
        output: {
          path: options.outDir
        },
        input: {
          index: options.input
        }
      },
      configPath
    };

    const { watch } = await resolveCore();
    handleAsyncOperationErrors(
      watch(defaultOptions),
      'error during watch project'
    );
  });

cli
  .command('preview', 'compile the project in watch mode')
  .option('--port <port>', 'specify port')
  .option('--open', 'open browser on server preview start')
  .action(async (options: FarmCLIPreviewOptions & GlobalFarmCLIOptions) => {
    const configPath = getConfigPath(options.config);
    const resolveOptions = resolveCommandOptions(options);
    const defaultOptions = {
      mode: options.mode,
      server: resolveOptions,
      configPath
    };

    const { preview } = await resolveCore();
    handleAsyncOperationErrors(
      preview(defaultOptions),
      'Failed to start preview server'
    );
  });

cli
  .command('clean [path]', 'Clean up the cache built incrementally')
  .option(
    '--recursive',
    'Recursively search for node_modules directories and clean them'
  )
  .action(async (cleanPath: string, options: ICleanOptions) => {
    const rootPath = cleanPath
      ? path.resolve(process.cwd(), cleanPath)
      : process.cwd();
    const { clean } = await resolveCore();

    try {
      await clean(rootPath, options?.recursive);
    } catch (e) {
      const { DefaultLogger } = await import('@farmfe/core');
      const logger = new DefaultLogger();
      logger.error(`Failed to clean cache:\n${e.stack}`);
      process.exit(1);
    }
  });

// create plugins command
cli
  .command('plugin [command]', 'Commands for manage plugins', {
    allowUnknownOptions: true
  })
  .action(async (command: keyof typeof COMMANDS, args: unknown) => {
    try {
      COMMANDS[command](args);
    } catch (e) {
      const { DefaultLogger } = await import('@farmfe/core');
      const logger = new DefaultLogger();
      logger.error(
        `The command arg parameter is incorrect. If you want to create a plugin in farm. such as "farm plugin create"\n${e.stack}`
      );
      process.exit(1);
    }
  });

// Listening for unknown command
cli.on('command:*', async () => {
  const { DefaultLogger } = await import('@farmfe/core');
  const logger = new DefaultLogger();
  logger.error(
    'Unknown command place Run "farm --help" to see available commands'
  );
});

// use mdn browser compatibility data with experimental warning in terminal so prevent experimental warning
preventExperimentalWarning();

cli.help();

cli.version(version);

cli.parse();
