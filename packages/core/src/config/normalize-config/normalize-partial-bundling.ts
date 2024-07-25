import { CUSTOM_KEYS } from '../constants.js';
import { ResolvedCompilation } from '../types.js';

export default function normalizePartialBundling(
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
