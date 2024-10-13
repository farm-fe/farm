import type { Resource } from '@farmfe/runtime/src/resource-loader.js';

export function getDynamicResources(
  dynamicResourcesMap: Record<string, string[][]> | null
): {
  dynamicResources: Resource[] | null;
  dynamicModuleResourcesMap: Record<string, number[]> | null;
} {
  let dynamicResources: Resource[] | null = null;
  let dynamicModuleResourcesMap: Record<string, number[]> | null = null;
  let visitedMap = new Map();

  if (dynamicResourcesMap) {
    dynamicResources = [];
    dynamicModuleResourcesMap = {};

    for (const [key, value] of Object.entries(dynamicResourcesMap)) {
      for (const r of value) {
        const visitedKey = r[0] + '.' + r[1];
        if (visitedMap.has(visitedKey)) {
          dynamicModuleResourcesMap[key] ??= [];
          dynamicModuleResourcesMap[key].push(visitedMap.get(visitedKey));
          continue;
        }
        dynamicResources.push({
          path: r[0],
          type: r[1] === 'script' ? 0 : 1
        });

        dynamicModuleResourcesMap[key] ??= [];
        dynamicModuleResourcesMap[key].push(dynamicResources.length - 1);
        visitedMap.set(visitedKey, dynamicResources.length - 1);
      }
    }
  }

  return { dynamicResources, dynamicModuleResourcesMap };
}
