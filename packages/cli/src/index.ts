import {
  VERSION as CORE_VERSION,
  FarmCliOptions,
  UserConfig
} from '@farmfe/core';
import { cac } from 'cac';
import { normalizeCliArgv } from './argv.js';
import { resolveSsrRunOptions } from './ssr.js';
import type {
  CleanOptions,
  CliBuildOptions,
  CliPreviewOptions,
  CliServerOptions,
  CliSsrOptions,
  GlobalCliOptions
} from './types.js';
import {
  handleAsyncOperationErrors,
  resolveCliConfig,
  resolveCommandOptions,
  resolveCore,
  resolveSsr,
  VERSION
} from './utils.js';

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

      const defaultOptions: FarmCliOptions = {
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
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('-i, --input <file>', '[string] input file path')
  .option(
    '--target <target>',
    '[string] transpile targetEnv node, browser, library'
  )
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

// watch command
cli
  .command(
    'watch [root]',
    'compile the project in development, watch file changes and rebuild when file changes'
  )
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('-i, --input <file>', '[string] input file path')
  .option(
    '--target <target>',
    '[string] transpile targetEnv node, browser, library'
  )
  .option('--format <format>', '[string] transpile format esm, commonjs')
  .option('--sourcemap', '[boolean] output source maps for build')
  .action(async (root: string, options: CliBuildOptions & GlobalCliOptions) => {
    const defaultOptions = {
      root,
      configFile: options.config,
      mode: options.mode,
      compilation: {
        output: {
          path: options?.outDir,
          targetEnv: options?.target,
          format: options?.format
        },
        input: {
          index: options?.input
        },
        sourcemap: options.sourcemap
      }
    };
    const { watch } = await resolveCore();

    handleAsyncOperationErrors(watch(defaultOptions), 'error during build');
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
      const defaultOptions: FarmCliOptions & UserConfig = {
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

function createSsrAction(command: 'dev' | 'build' | 'preview') {
  return async (root = '', options: CliSsrOptions & GlobalCliOptions) => {
    const resolvedOptions = resolveSsrRunOptions({
      command,
      root,
      options
    });
    const { runSsrCommand } = await resolveSsr();

    handleAsyncOperationErrors(
      runSsrCommand(resolvedOptions),
      `Failed to run "farm ssr ${command}"`
    );
  };
}

cli
  .command('ssr dev [root]', 'run SSR toolkit in dev mode')
  .option('--client-config <file>', '[string] client config file')
  .option('--server-config <file>', '[string] server config file')
  .option('--entry <file>', '[string] server render entry file')
  .option('--export-name <name>', '[string] render export name')
  .option('--template-file <file>', '[string] html template file')
  .option('--template-resource <name>', '[string] template resource name')
  .option('--placeholder <html>', '[string] html placeholder')
  .option('--host <host>', '[string] specify host')
  .option('--port <port>', '[number] specify port')
  .option('--open', '[boolean] open browser on server start')
  .option('--strictPort', '[boolean] exit if specified port is already in use')
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('--base <path>', '[string] public base path')
  .action(createSsrAction('dev'));

cli
  .command('ssr build [root]', 'build SSR toolkit client and server bundles')
  .option('--client-config <file>', '[string] client config file')
  .option('--server-config <file>', '[string] server config file')
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('-i, --input <file>', '[string] input file path')
  .option(
    '--target <target>',
    '[string] transpile targetEnv node, browser, library'
  )
  .option('--format <format>', '[string] transpile format esm, commonjs')
  .option('--sourcemap', '[boolean] output source maps for build')
  .option(
    '--treeShaking',
    '[boolean] Eliminate useless code without side effects'
  )
  .option('--minify', '[boolean] code compression at build time')
  .option('--base <path>', '[string] public base path')
  .action(createSsrAction('build'));

cli
  .command('ssr preview [root]', 'preview SSR toolkit server from build output')
  .option('--client-config <file>', '[string] client config file')
  .option('--server-config <file>', '[string] server config file')
  .option('--entry <file>', '[string] server render entry file')
  .option('--export-name <name>', '[string] render export name')
  .option('--template-file <file>', '[string] html template file')
  .option('--placeholder <html>', '[string] html placeholder')
  .option('--host <host>', '[string] specify host')
  .option('--port <port>', '[number] specify port')
  .option('--open', '[boolean] open browser on preview server start')
  .option('--strictPort', '[boolean] exit if specified port is already in use')
  .option('-o, --outDir <dir>', '[string] output directory')
  .option('--base <path>', '[string] public base path')
  .action(createSsrAction('preview'));

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

cli.parse(normalizeCliArgv(process.argv));
