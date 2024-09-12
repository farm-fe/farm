// @ts-ignore
import React from 'react';

function isClassComponent(v: any): boolean {
// class component can not be hot-updated, so skip it and find its parent
  if(v && v.prototype instanceof React.Component) {
    return true;
  }

  return false;
}

export default function isReactRefreshBoundary(Refresh: any, moduleExports: any): boolean {

  if (Refresh.isLikelyComponentType(moduleExports)) {
    if(isClassComponent(moduleExports)) return false;
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

    if(Refresh.isLikelyComponentType(exportValue)) {
      if(isClassComponent(exportValue)) return false;
    }else {
      areAllExportsComponents = false;
    }
  }
  return hasExports && areAllExportsComponents;
};