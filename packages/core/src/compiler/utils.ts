import { ResolvedUserConfig } from '../config/types.js';
import { getPluginHooks } from '../plugin/index.js';
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
  for (const hook of getPluginHooks(config.jsPlugins, 'configureCompiler')) {
    await hook?.(compiler);
  }
}

export async function createInlineCompiler(
  config: ResolvedUserConfig,
  options = {}
) {
  const { Compiler } = await import('./index.js');
  return new Compiler({
    config: { ...config.compilation, ...options },
    jsPlugins: config.jsPlugins,
    rustPlugins: config.rustPlugins
  });
}
