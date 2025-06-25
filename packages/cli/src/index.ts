import { readFileSync } from 'node:fs';

import { cac } from 'cac';
import { getOptionFromBuildOption } from './config.js';
import {
  handleAsyncOperationErrors,
  preventExperimentalWarning,
  resolveCliConfig,
  resolveCommandOptions,
  resolveCore
} from './utils.js';

import { FarmCLIOptions } from '@farmfe/core';
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
  .alias('dev')
  .option('-l, --lazy', 'lazyCompilation')
  .option('--host <host>', 'specify host')
  .option('--port <port>', 'specify port')
  .option('--open', 'open browser on server start')
  .option('--hmr', 'enable hot module replacement')
  .option('--cors', 'enable cors')
  .option('--strictPort', 'specified port is already in use, exit with error')
  .action(
    async (
      rootPath: string,
      options: FarmCLIServerOptions & GlobalFarmCLIOptions
    ) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);
      const resolveOptions = resolveCommandOptions(options);

      const defaultOptions: FarmCLIOptions = {
        root,
        compilation: {
          lazyCompilation: options.lazy
        },
        server: resolveOptions,
        clearScreen: options.clearScreen,
        configPath,
        mode: options.mode
      };

      if (options.base) {
        defaultOptions.compilation.output = {
          publicPath: options.base
        };
      }

      const { start } = await resolveCore();
      handleAsyncOperationErrors(
        start(defaultOptions),
        'Failed to start server'
      );
    }
  );

// build command
cli
  .command('build [root]', 'compile the project in production mode')
  .option('-o, --outDir <dir>', 'output directory')
  .option('-i, --input <file>', 'input file path')
  .option('-w, --watch', 'watch file change')
  .option('--target <target>', 'transpile targetEnv node, browser')
  .option('--format <format>', 'transpile format esm, commonjs')
  .option('--sourcemap', 'output source maps for build')
  .option('--treeShaking', 'Eliminate useless code without side effects')
  .option('--minify', 'code compression at build time')
  .action(
    async (
      rootPath: string,
      options: FarmCLIBuildOptions & GlobalFarmCLIOptions
    ) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);

      const defaultOptions = {
        root,
        configPath,
        ...getOptionFromBuildOption(options)
      };

      const { build } = await resolveCore();
      handleAsyncOperationErrors(build(defaultOptions), 'error during build');
    }
  );

cli
  .command('watch [root]', 'watch file change')
  .option('-o, --outDir <dir>', 'output directory')
  .option('-i, --input <file>', 'input file path')
  .option('--target <target>', 'transpile targetEnv node, browser')
  .option('--format <format>', 'transpile format esm, commonjs')
  .option('--sourcemap', 'output source maps for build')
  .option('--treeShaking', 'Eliminate useless code without side effects')
  .option('--minify', 'code compression at build time')
  .action(
    async (
      rootPath: string,
      options: FarmCLIBuildOptions & GlobalFarmCLIOptions
    ) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);

      const defaultOptions = {
        root,
        configPath,
        ...getOptionFromBuildOption(options)
      };

      const { watch } = await resolveCore();
      handleAsyncOperationErrors(
        watch(defaultOptions),
        'error during watch project'
      );
    }
  );

cli
  .command('preview [root]', 'compile the project in watch mode')
  .option('--port <port>', 'specify port')
  .option('--open', 'open browser on server preview start')
  .action(
    async (
      rootPath: string,
      options: FarmCLIPreviewOptions & GlobalFarmCLIOptions
    ) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);

      const resolveOptions = resolveCommandOptions(options);
      const defaultOptions: FarmCLIOptions = {
        root,
        mode: options.mode,
        server: resolveOptions,
        configPath,
        port: options.port,
        clearScreen: options.clearScreen
      };

      if (options.base) {
        defaultOptions.compilation.output = {
          publicPath: options.base
        };
      }

      const { preview } = await resolveCore();
      handleAsyncOperationErrors(
        preview(defaultOptions),
        'Failed to start preview server'
      );
    }
  );

cli
  .command('clean [path]', 'Clean up the cache built incrementally')
  .option(
    '--recursive',
    'Recursively search for node_modules directories and clean them'
  )
  .action(async (rootPath: string, options: ICleanOptions) => {
    const { root } = resolveCliConfig(rootPath, options);
    const { clean } = await resolveCore();

    try {
      await clean(root, options?.recursive);
    } catch (e) {
      const { Logger } = await import('@farmfe/core');
      const logger = new Logger();
      logger.error(`Failed to clean cache: \n ${e.stack}`);
      process.exit(1);
    }
  });

// Listening for unknown command
cli.on('command:*', async () => {
  const { Logger } = await import('@farmfe/core');
  const logger = new Logger();
  logger.error(
    'Unknown command place Run "farm --help" to see available commands'
  );
});

// warning::: use mdn browser compatibility data with experimental warning in terminal so prevent experimental warning
// we don't use it in `@farmfe/core` package because
// we need to prevent it in cli package but we don't prevent it in core package
// We only keep the original code environment.
preventExperimentalWarning();

cli.help();

cli.version(version);

cli.parse();
