import { readFileSync } from 'node:fs';

import { cac } from 'cac';
import { getOptionFromBuildOption } from './config.js';
import {
  cleanOptions,
  filterDuplicateOptions,
  handleAsyncOperationErrors,
  resolveCliConfig,
  resolveCommandOptions,
  resolveCore
} from './utils.js';

import type {
  CleanOptions,
  CliBuildOptions,
  CliPreviewOptions,
  CliServerOptions,
  GlobalCliOptions
} from './types.js';

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

const cli = cac('farm');

// common command
cli
  .option(
    '-c, --config <file>',
    '[string] use specified config file (default: farm.config.js / farm.config.ts / farm.config.mjs / farm.config.cjs / farm.config.mts / farm.config.cts)'
  )
  .option(
    '-m, --mode <mode>',
    '[string] set env mode, when use with development (default: /)'
  )
  .option('--base <path>', '[string] public base path')
  .option('-d, --debug [feat]', `[string | boolean] show debug logs`)
  .option(
    '--clearScreen',
    '[boolean] allow/disable clear screen when logging (default: true)',
    {
      default: true
    }
  );

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
    async (rootPath: string, options: CliServerOptions & GlobalCliOptions) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);
      const resolveOptions = resolveCommandOptions(options);

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

      console.log(defaultOptions);

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
    async (rootPath: string, options: CliBuildOptions & GlobalCliOptions) => {
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
    async (rootPath: string, options: CliBuildOptions & GlobalCliOptions) => {
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
    async (rootPath: string, options: CliPreviewOptions & GlobalCliOptions) => {
      const { root, configPath } = resolveCliConfig(rootPath, options);

      const resolveOptions = resolveCommandOptions(options);
      const defaultOptions = {
        root,
        mode: options.mode,
        server: resolveOptions,
        configPath,
        port: options.port
      };

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
  .action(async (rootPath: string, options: CleanOptions) => {
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

cli
  .command(
    '[root]',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .alias('come')
  .alias('dev')
  .option('-l, --lazy', '[boolean] lazyCompilation (default: true)')
  .option('--host <host>', '[string] specify host')
  .option('--port <port>', '[string] specify port')
  .option('--open', '[boolean] open browser on server start')
  .option('--hmr', '[boolean] enable hot module replacement')
  .option('--cors', '[boolean] enable cors')
  .option(
    '--strictPort',
    '[boolean] specified port is already in use, exit with error (default: true)'
  )
  .action(
    async (root: string, options: CliServerOptions & GlobalCliOptions) => {
      filterDuplicateOptions(options);

      const defaultOptions = {
        root,
        compilation: {
          lazyCompilation: options.lazy
        },
        server: cleanOptions(options),
        clearScreen: options.clearScreen,
        configFile: options.config,
        mode: options.mode
      };

      const { startTestRefactorCli } = await resolveCore();
      handleAsyncOperationErrors(
        startTestRefactorCli(defaultOptions),
        'Failed to start server'
      );
    }
  );

cli.help();

cli.version(version);

cli.parse();
