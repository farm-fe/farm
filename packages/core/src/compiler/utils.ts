import { ResolvedUserConfig } from '../config/types.js';
import { Compiler } from './index.js';

export function createCompiler(resolvedUserConfig: ResolvedUserConfig) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig,
    logger
  } = resolvedUserConfig;

  const compiler = new Compiler(
    {
      config: compilationConfig,
      jsPlugins,
      rustPlugins
    },
    logger
  );
  return compiler;
}

export function resolveConfigureCompilerHook(config: ResolvedUserConfig) {
  console.log(config.jsPlugins);
  // for (const plugin of resolvedUserConfig.jsPlugins) {
  //   await plugin.configureCompiler?.(compiler);
  // }
}
