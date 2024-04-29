import { FarmCLIOptions, UserConfig } from '@farmfe/core';
import { FarmCLIBuildOptions, GlobalFarmCLIOptions } from './types.js';

export function getOptionFromBuildOption(
  options: FarmCLIBuildOptions & GlobalFarmCLIOptions
): FarmCLIOptions & UserConfig {
  const input: Record<string, string> = {};
  if (options?.input) {
    input.index = options.input;
  }
  const output: UserConfig['compilation']['output'] = {};

  if (options.outDir) {
    output.path = options.outDir;
  }
  if (options.target) {
    output.targetEnv = options.target;
  }
  if (options.format) {
    output.format = options.format;
  }

  const compilation: UserConfig['compilation'] = { input, output };

  if (typeof options?.watch === 'boolean') {
    compilation.watch = options.watch;
  }

  if (options.minify) {
    compilation.minify = options.minify;
  }
  if (options.sourcemap) {
    compilation.sourcemap = options.sourcemap;
  }

  if (options.treeShaking) {
    compilation.treeShaking = options.treeShaking;
  }

  const defaultOptions = {
    compilation,
    mode: options.mode
  };

  return defaultOptions;
}
