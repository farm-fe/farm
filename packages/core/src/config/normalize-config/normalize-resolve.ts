import { CUSTOM_KEYS } from '../constants.js';
import { ResolvedCompilation, UserConfig } from '../types.js';

export function normalizeResolve(
  config: UserConfig,
  resolvedCompilation: ResolvedCompilation
) {
  let dedupe: string[] = [
    ...(config?.compilation?.custom[CUSTOM_KEYS.resolve_dedupe] ?? [])
  ];

  if (config?.compilation?.resolve?.dedupe) {
    dedupe = config.compilation.resolve.dedupe;
    delete config.compilation.resolve.dedupe;
  }

  resolvedCompilation.custom[CUSTOM_KEYS.resolve_dedupe] =
    JSON.stringify(dedupe);
}
