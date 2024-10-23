import { CUSTOM_KEYS } from '../constants.js';
import { ResolvedCompilation, UserConfig } from '../types.js';

export function normalizeAsset(
  config: UserConfig,
  resolvedCompilation: ResolvedCompilation
) {
  if (config.compilation?.assets?.mode) {
    const mode = config.compilation.assets.mode;

    // biome-ignore lint/style/noNonNullAssertion: <explanation>
    resolvedCompilation.custom![CUSTOM_KEYS.assets_mode] = JSON.stringify(mode);
  }
}
