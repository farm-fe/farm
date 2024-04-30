import { FarmCLIOptions, UserConfig } from '@farmfe/core';
import { FarmCLIBuildOptions, GlobalFarmCLIOptions } from './types.js';

export function getOptionFromBuildOption(
  options: FarmCLIBuildOptions & GlobalFarmCLIOptions
): FarmCLIOptions & UserConfig {
  const {
    input,
    outDir,
    target,
    format,
    watch,
    minify,
    sourcemap,
    treeShaking,
    mode
  } = options;

  const output: UserConfig['compilation']['output'] = {
    ...(outDir && { path: outDir }),
    ...(target && { targetEnv: target }),
    ...(format && { format })
  };

  const compilation: UserConfig['compilation'] = {
    input: { ...(input && { index: input }) },
    output,
    ...(watch && { watch }),
    ...(minify && { minify }),
    ...(sourcemap && { sourcemap }),
    ...(treeShaking && { treeShaking })
  };

  const defaultOptions: FarmCLIOptions & UserConfig = {
    compilation,
    ...(mode && { mode })
  };

  return defaultOptions;
}
