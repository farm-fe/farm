import { readFileSync } from 'node:fs';
import { cac } from 'cac';
import { DefaultLogger } from '@farmfe/core';
import { resolveCore, getConfigPath, resolveCommandOptions } from './utils.js';
import { COMMANDS } from './plugin/index.js';

import type {
  FarmCLIBuildOptions,
  FarmCLIPreviewOptions,
  FarmCLIServerOptions,
  GlobalFarmCLIOptions
} from './types.js';
import path from 'node:path';

const logger = new DefaultLogger();

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

const cli = cac('farm');

// common command
cli
  .option('-c, --config <file>', 'use specified config file')
  .option('-m, --mode <mode>', 'set env mode')
  .option('--base <path>', 'public base path')
  .option('--clearScreen', 'allow/disable clear screen when logging');

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

      const defaultOptions = {
        root,
        compilation: {
          lazyCompilation: options.lazy
        },
        server: resolveOptions,
        clearScreen: options.clearScreen ?? true,
        configPath,
        mode: options.mode
      };

      const { start } = await resolveCore();

      try {
        await start(defaultOptions);
      } catch (e) {
        logger.error(`Failed to start server:\n ${e.stack}`);
        process.exit(1);
      }
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
    console.log(defaultOptions);

    const { build } = await resolveCore();

    try {
      build(defaultOptions);
    } catch (e) {
      logger.error(`error during build:\n${e.stack}`);
      process.exit(1);
    }
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

    try {
      watch(defaultOptions);
    } catch (e) {
      logger.error(`error during watch project:\n${e.stack}`);
      process.exit(1);
    }
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

    try {
      preview(defaultOptions);
    } catch (e) {
      logger.error(`Failed to start preview server:\n${e.stack}`);
      process.exit(1);
    }
  });

interface ICleanOptions {
  path?: string;
  recursive?: boolean;
}

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
      logger.error(`Failed to clean cache:\n${e.stack}`);
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
