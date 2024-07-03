import { readFileSync } from 'node:fs';

import { defineCommand, runMain } from 'citty';
import { getOptionFromBuildOption } from './config.js';
import {
  handleAsyncOperationErrors,
  preventExperimentalWarning,
  resolveCliConfig,
  resolveCommandOptions,
  resolveCore
} from './utils.js';

import type {
  FarmCLIBuildOptions,
  FarmCLICommonOptions,
  FarmCLIPreviewOptions,
  FarmCLIServerOptions,
  GlobalFarmCLIOptions,
  ICleanOptions,
  NormalizedFarmCLIBuildOptions
} from './types.js';

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

const devCommand = defineCommand({
  meta: {
    name: 'dev',
    description:
      'Compile the project in dev mode and serve it with farm dev server'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    lazy: { type: 'boolean', alias: 'l', description: 'lazyCompilation' },
    host: { type: 'string', description: 'specify host' },
    port: { type: 'string', description: 'specify port' },
    open: { type: 'boolean', description: 'open browser on server start' },
    hmr: { type: 'boolean', description: 'enable hot module replacement' },
    cors: { type: 'boolean', description: 'enable cors' },
    strictPort: {
      type: 'boolean',
      description: 'specified port is already in use, exit with error'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & FarmCLIServerOptions }) {
    const { root, configPath } = resolveCliConfig(args.root, args.config);

    const resolvedOptions = resolveCommandOptions(args as GlobalFarmCLIOptions);
    const defaultOptions = {
      root,
      compilation: {
        lazyCompilation: args.lazy
      },
      server: resolvedOptions,
      clearScreen: args.clearScreen,
      configPath,
      mode: args.mode
    };
    const { start } = await resolveCore();
    handleAsyncOperationErrors(start(defaultOptions), 'Failed to start server');
  }
});

const buildCommand = defineCommand({
  meta: {
    name: 'build',
    description: 'compile the project in production mode'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    outDir: {
      type: 'string',
      alias: 'o',
      description: 'output directory'
    },
    input: {
      type: 'string',
      alias: 'i',
      description: 'input file path'
    },
    watch: { type: 'boolean', alias: 'w', description: 'watch file change' },
    target: {
      type: 'string',
      description: 'transpile targetEnv node, browser'
    },
    format: {
      type: 'string',
      description: 'transpile format esm, commonjs'
    },
    sourcemap: {
      type: 'boolean',
      description: 'output source maps for build'
    },
    treeShaking: {
      type: 'boolean',
      description: 'Eliminate useless code without side effects'
    },
    minify: {
      type: 'boolean',
      description: 'code compression at build time'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & FarmCLIBuildOptions }) {
    const { root, configPath } = resolveCliConfig(
      args.root,
      args.config ?? args.c
    );

    const defaultOptions = {
      root,
      configPath,
      ...getOptionFromBuildOption(args as NormalizedFarmCLIBuildOptions)
    };

    const { build } = await resolveCore();
    handleAsyncOperationErrors(build(defaultOptions), 'error during build');
  }
});

const watchCommand = defineCommand({
  meta: {
    name: 'watch',
    description: 'watch file change'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    outDir: {
      type: 'string',
      alias: 'o',
      description: 'output directory'
    },
    input: {
      type: 'string',
      alias: 'i',
      description: 'input file path'
    },
    target: {
      type: 'string',
      description: 'transpile targetEnv node, browser'
    },
    format: {
      type: 'string',
      description: 'transpile format esm, commonjs'
    },
    sourcemap: {
      type: 'boolean',
      description: 'output source maps for build'
    },
    treeShaking: {
      type: 'boolean',
      description: 'Eliminate useless code without side effects'
    },
    minify: {
      type: 'boolean',
      description: 'code compression at build time'
    }
  },
  async run({ args }: { args: FarmCLIBuildOptions & GlobalFarmCLIOptions }) {
    const { root, configPath } = resolveCliConfig(
      args.root,
      args.config ?? args.c
    );

    const defaultOptions = {
      root,
      configPath,
      ...getOptionFromBuildOption(args as NormalizedFarmCLIBuildOptions)
    };

    const { watch } = await resolveCore();
    handleAsyncOperationErrors(
      watch(defaultOptions),
      'error during watch project'
    );
  }
});

const previewCommand = defineCommand({
  meta: {
    name: 'preview',
    description: 'compile the project in watch mode'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    port: { type: 'string', description: 'specify port' },
    open: {
      type: 'boolean',
      description: 'open browser on server preview start'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & FarmCLIPreviewOptions }) {
    const { root, configPath } = resolveCliConfig(args.root, args.config);

    const resolvedOptions = resolveCommandOptions(args as GlobalFarmCLIOptions);
    const defaultOptions = {
      root,
      mode: args.mode,
      server: resolvedOptions,
      configPath,
      port: resolvedOptions.port
    };

    const { preview } = await resolveCore();
    handleAsyncOperationErrors(
      preview(defaultOptions),
      'Failed to start preview server'
    );
  }
});

const cleanCommand = defineCommand({
  meta: {
    name: 'clean',
    description: 'Clean up the cache built incrementally'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    recursive: {
      type: 'boolean',
      alias: 'r',
      description:
        'Recursively search for node_modules directories and clean them'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & ICleanOptions }) {
    const { root } = resolveCliConfig(args.root, args.config);

    const { clean } = await resolveCore();
    try {
      await clean(root, args.recursive);
    } catch (e) {
      const { Logger } = await import('@farmfe/core');
      const logger = new Logger();
      logger.error(`Failed to clean cache: \n ${e.stack}`);
      process.exit(1);
    }
  }
});

const main = defineCommand({
  meta: {
    name: 'farm',
    version
  },
  args: {
    config: {
      type: 'string',
      alias: 'c',
      description: 'use specified config file'
    },
    mode: { type: 'string', alias: 'm', description: 'set env mode' },
    base: { type: 'string', description: 'public base path' },
    clearScreen: {
      type: 'boolean',
      default: true,
      description: 'allow/disable clear screen when logging'
    }
  },
  subCommands: {
    dev: devCommand,
    // alias for dev
    start: devCommand,
    build: buildCommand,
    watch: watchCommand,
    preview: previewCommand,
    clean: cleanCommand
  }
});

// warning::: use mdn browser compatibility data with experimental warning in terminal so prevent experimental warning
// we don't use it in `@farmfe/core` package because
// we need to prevent it in cli package but we don't prevent it in core package
// We only keep the original code environment.
preventExperimentalWarning();

// default to start a development server
if (process.argv.slice(2).length === 0)
  runMain(main, { rawArgs: process.argv.slice(2).concat(['dev']) });
else runMain(main);
