import { ResolvedUserConfig } from '../config/types.js';
import { getPluginHooks, getSortedPlugins } from '../plugin/index.js';
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

export async function resolveConfigureCompilerHook(
  compiler: Compiler,
  config: ResolvedUserConfig
) {
  for (const plugin of getPluginHooks(config.jsPlugins, 'configureCompiler')) {
    await plugin.configureCompiler?.(compiler);
  }
}
