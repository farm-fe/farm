import { CUSTOM_KEYS } from '../constants.js';

import type { ResolvedCompilation } from '../types.js';

export function normalizePartialBundling(
  resolvedCompilation: ResolvedCompilation
) {
  const partialBundlingItemEnforceMap: Record<string, boolean> = {};
  const partialBundleGroups = resolvedCompilation.partialBundling?.groups ?? [];

  for (const group of partialBundleGroups) {
    if (group.enforce) {
      partialBundlingItemEnforceMap[group.name] = true;
    }
  }

  resolvedCompilation.custom[CUSTOM_KEYS.partial_bundling_groups_enforce] =
    JSON.stringify(partialBundlingItemEnforceMap);
}
