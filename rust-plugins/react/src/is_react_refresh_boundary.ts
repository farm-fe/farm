export default function isReactRefreshBoundary(Refresh, moduleExports): boolean {
  if (Refresh.isLikelyComponentType(moduleExports)) {
    return true;
  }

  if (moduleExports == null || typeof moduleExports !== 'object') {
    // Exit if we can't iterate over exports.
    return false;
  }

  let hasExports = false;
  let areAllExportsComponents = true;

  for (const key in moduleExports) {
    hasExports = true;
    if (key === '__esModule') {
      continue;
    }
    
    const exportValue = moduleExports[key];
    if (!Refresh.isLikelyComponentType(exportValue)) {
      areAllExportsComponents = false;
    }
  }
  return hasExports && areAllExportsComponents;
};