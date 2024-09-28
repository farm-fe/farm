import { CUSTOM_KEYS } from '../constants.js';
import { ResolvedCompilation, UserConfig } from '../types.js';

export function normalizeCss(
  config: UserConfig,
  resolvedCompilation: ResolvedCompilation
) {
  if (config.compilation?.css?.modules) {
    normalizeCssModules(config, resolvedCompilation);
  }
}

function normalizeCssModules(
  config: UserConfig,
  resolvedCompilation: ResolvedCompilation
) {
  if (config.compilation.css.modules.localsConversion) {
    const localsConvention = config.compilation.css.modules.localsConversion;
    delete resolvedCompilation.css.modules.localsConversion;
    if (typeof localsConvention === 'string') {
      resolvedCompilation.custom[CUSTOM_KEYS.css_locals_conversion] =
        JSON.stringify(localsConvention);
    }
  }
}
