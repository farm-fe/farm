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

  const alias = normalizeResolveAlias(config);

  if (alias.length) {
    resolvedCompilation.resolve.alias = alias;
  }
}

export function normalizeResolveAlias(
  config: any
): Array<{ find: string | RegExp; replacement: string }> {
  const alias = config.compilation.resolve?.alias;
  const logger = config.logger;

  const normalizeItem = (find: string | RegExp, replacement: string) => {
    if (typeof find === 'string' || find instanceof RegExp) {
      if (typeof replacement === 'string') {
        return { find, replacement };
      }
      logger.warn(`Invalid replacement for '${find}': must be a string`);
    } else {
      logger.warn(`Invalid alias key: '${find}' must be a string or RegExp`);
    }
    return null;
  };

  const normalizeArray = (arr: any) =>
    arr
      .map((item: any, _index: any) =>
        typeof item === 'object' && item !== null
          ? normalizeItem(item.find, item.replacement)
          : normalizeItem(item, item)
      )
      .filter(Boolean);

  const normalizeObject = (obj: any) => {
    return Object.entries(obj as Record<string, string>)
      .map(([find, replacement]) => {
        const result = normalizeItem(find, replacement);
        return result;
      })
      .filter(Boolean);
  };

  let result: Array<{ find: string | RegExp; replacement: string }>;
  switch (true) {
    case alias === null || alias === undefined:
      result = [];
      break;
    case Array.isArray(alias):
      result = normalizeArray(alias);
      break;
    case typeof alias === 'object':
      result = normalizeObject(alias);
      break;
    default:
      logger.warn('Alias configuration must be an object or an array');
      result = [];
  }
  return result;
}
