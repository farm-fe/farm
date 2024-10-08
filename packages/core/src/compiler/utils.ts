import { Compiler } from './index.js';

import type {
  ResolvedCompilation,
  ResolvedUserConfig
} from '../config/types.js';

export function createCompiler(resolvedUserConfig: ResolvedUserConfig) {
  return new Compiler(resolvedUserConfig);
}

export function createInlineCompiler(
  config: ResolvedUserConfig,
  options: ResolvedCompilation = {}
) {
  return new Compiler({
    ...config,
    compilation: { ...config.compilation, ...options }
  });
}
