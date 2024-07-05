import { defineCommand } from 'citty';
import { getOptionFromBuildOption } from '../config.js';
import {
  FarmCLIBuildOptions,
  GlobalFarmCLIOptions,
  NormalizedFarmCLIBuildOptions
} from '../types.js';
import {
  handleAsyncOperationErrors,
  resolveCliConfig,
  resolveCore
} from '../utils.js';

export default defineCommand({
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
