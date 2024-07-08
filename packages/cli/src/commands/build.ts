import { defineCommand } from 'citty';
import { getOptionFromBuildOption } from '../config.js';
import {
  FarmCLIBuildOptions,
  FarmCLICommonOptions,
  NormalizedFarmCLIBuildOptions
} from '../types.js';
import {
  handleAsyncOperationErrors,
  resolveCliConfig,
  resolveCore
} from '../utils.js';

export default defineCommand({
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
