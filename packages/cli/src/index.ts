import { VERSION as CORE_VERSION } from '@farmfe/core';
import { cac } from 'cac';

import {
  VERSION,
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
  .option('--target <target>', '[string] transpile targetEnv node, browser')
  .option('--format <format>', '[string] transpile format esm, commonjs')
  .option('--sourcemap', '[boolean] output source maps for build')
  .option(
    '--treeShaking',
    '[boolean] Eliminate useless code without side effects'
  )
  .option('--minify', '[boolean] code compression at build time')
  .action(
    async (
      root: string,
      options: CliServerOptions & CliBuildOptions & GlobalCliOptions
    ) => {
      const resolveOptions = resolveCommandOptions(options);

      const defaultOptions = {
        root,
        server: resolveOptions,
        clearScreen: options.clearScreen,
        configFile: options.config,
        mode: options.mode,
        compilation: {
          lazyCompilation: options.lazy,
          output: {
            path: options?.outDir,
            targetEnv: options?.target,
            format: options?.format
          },
          input: {
            index: options?.input
          },
          sourcemap: options.sourcemap,
          minify: options.minify,
          treeShaking: options.treeShaking
        }
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
  .command('build [root]', 'compile the project in production mode')
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('-i, --input <file>', '[string] input file path')
  .option('-w, --watch', '[boolean] watch file change and rebuild')
  .option('--target <target>', '[string] transpile targetEnv node, browser')
  .option('--format <format>', '[string] transpile format esm, commonjs')
  .option('--sourcemap', '[boolean] output source maps for build')
  .option(
    '--treeShaking',
    '[boolean] Eliminate useless code without side effects'
  )
  .option('--minify', '[boolean] code compression at build time')
  .action(async (root: string, options: CliBuildOptions & GlobalCliOptions) => {
    const defaultOptions = {
      root,
      configFile: options.config,
      mode: options.mode,
      watch: options.watch,
      compilation: {
        output: {
          path: options?.outDir,
          targetEnv: options?.target,
          format: options?.format
        },
        input: {
          index: options?.input
        },
        sourcemap: options.sourcemap,
        minify: options.minify,
        treeShaking: options.treeShaking
      }
    };
    const { build } = await resolveCore();

    handleAsyncOperationErrors(build(defaultOptions), 'error during build');
  });

cli
  .command('preview [root]', 'compile the project in watch mode')
  .option('--host [host]', `[string] specify hostname`)
  .option('--port <port>', `[number] specify port`)
  .option('--open', '[boolean] open browser on server preview start')
  .option('--outDir <dir>', `[string] output directory (default: dist)`)
  .option('--strictPort', `[boolean] exit if specified port is already in use`)
  .action(
    async (root: string, options: CliPreviewOptions & GlobalCliOptions) => {
      const defaultOptions = {
        root,
        mode: options.mode,
        server: {
          preview: {
            host: options.host,
            port: options.port,
            open: options.open,
            strictPort: options.strictPort,
            distDir: options.outDir
          }
        },
        configFile: options.config,
        port: options.port,
        compilation: {
          output: {
            path: options.outDir
          }
        }
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
    handleAsyncOperationErrors(
      clean(root, options?.recursive),
      'Failed to clean cache'
    );
  });

cli.help();

cli.version(
  `@farmfe/cli ${VERSION ?? 'unknown'} @farmfe/core ${CORE_VERSION ?? 'unknown'}`
);

cli.parse();
