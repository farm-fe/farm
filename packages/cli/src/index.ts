import { cac } from 'cac';
import { readFileSync } from 'node:fs';
import Module from 'node:module';
import { pathToFileURL } from 'node:url';
import { DefaultLogger, isWindows } from '@farmfe/core';
import { getConfigPath, resolveCommandOptions } from './utils.js';
import { COMMANDS } from './plugin/index.js';

import type { start, build, preview, watch } from '@farmfe/core';
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
  .option('--cors', 'enable CORS')
  // TODO add https config
  // .option('--https', 'use https')
  // TODO add strictPort config
  // .option('--strictPort', 'specified port is already in use, exit with error')
  .action(
    async (
      root: string,
      options: FarmCLIServerOptions & GlobalFarmCLIOptions
    ) => {
      const resolveOptions = resolveCommandOptions(options);
      const configPath = getConfigPath(options.config);
      const defaultOptions = {
        compilation: {
          root,
          mode: options.mode,
          lazyCompilation: options.lazy
        },
        server: resolveOptions,
        clearScreen: options.clearScreen ?? true,
        configPath
      };
      const { start } = await resolveCore(configPath);

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
  // TODO add target config esm, commonjs
  // .option("--target <target>", "transpile target")
  .option('--format <format>', 'transpile format esm, commonjs')
  .option('-o, --outDir <dir>', 'output directory')
  .option('-i, --input <file>', 'input file path')
  .option('--sourcemap', 'output source maps for build')
  .option('--treeShaking', 'Eliminate useless code without side effects')
  .option('--minify', 'code compression at build time')
  .option('-w, --watch', 'watch file change')
  .action(async (options: FarmCLIBuildOptions & GlobalFarmCLIOptions) => {
    const configPath = getConfigPath(options.config);
    const defaultOptions = {
      compilation: {
        mode: options.mode,
        watch: options.watch,
        output: options.outDir
          ? {
              path: options.outDir
            }
          : undefined,
        input: options.input
          ? {
              index: options.input
            }
          : undefined,
        sourcemap: options.sourcemap,
        minify: options.minify,
        treeShaking: options.treeShaking
      },
      configPath
    };
    const { build } = await resolveCore(configPath);

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
    const { watch } = await resolveCore(configPath);

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
      compilation: {
        mode: options.mode
      },
      server: resolveOptions,
      configPath
    };
    const { preview } = await resolveCore(configPath);

    try {
      preview(defaultOptions);
    } catch (e) {
      logger.error(`Failed to start preview server:\n${e.stack}`);
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

export function resolveCore(cwd: string): Promise<{
  start: typeof start;
  build: typeof build;
  watch: typeof watch;
  preview: typeof preview;
}> {
  const require = Module.createRequire(path.join(cwd, 'package.json'));

  let farmCorePath: string;
  try {
    farmCorePath = require.resolve('@farmfe/core');
  } catch (err) {
    // TODO Encapsulation logger
    console.error(
      `Cannot find @farmfe/core module, Did you successfully install: \n${err.stack},`
    );
    process.exit(1);
  }

  if (isWindows) {
    return import(pathToFileURL(farmCorePath).toString());
  }

  return import(farmCorePath);
}
