// @ts-ignore
import Refresh from 'react-refresh';

type ModExport = Record<string, any>;

// @ts-ignore
export function validateRefreshBoundaryAndEnqueueUpdate(prevExports: ModExport, nextExports: ModExport) {
  const ignoredExports = []
  if (
    predicateOnExport(
      ignoredExports,
      prevExports,
      (key) => key in nextExports,
    ) !== true
  ) {
    return 'Could not Fast Refresh (export removed)'
  }
  if (
    predicateOnExport(
      ignoredExports,
      nextExports,
      (key) => key in prevExports,
    ) !== true
  ) {
    return 'Could not Fast Refresh (new export)'
  }

  let hasExports = false
  const allExportsAreComponentsOrUnchanged = predicateOnExport(
    ignoredExports,
    nextExports,
    (key, value) => {
      hasExports = true
      if (Refresh.isLikelyComponentType(value)) return true
      return prevExports[key] === nextExports[key]
    },
  )

  if (hasExports && allExportsAreComponentsOrUnchanged === true) {
    enqueueUpdate()
  } else {
    return `Could not Fast Refresh ("${allExportsAreComponentsOrUnchanged}" export is incompatible).`
  }
}

function predicateOnExport(ignoredExports: string[], moduleExports: ModExport, predicate: (key: string, value: any) => boolean): string | true {
  for (const key in moduleExports) {
    if (key === '__esModule') continue
    if (ignoredExports.includes(key)) continue
    if (!predicate(key, moduleExports[key])) return key
  }
  return true
}

function debounce<Args extends unknown[]>(func: (...args: Args) => void, wait: number, immediate?: boolean) {
	var timeout: any;
	return function() {
		var context = this, args = arguments;
		var later = function() {
			timeout = null;
			if (!immediate) func.apply(context, args);
		};
		var callNow = immediate && !timeout;
		clearTimeout(timeout);
		timeout = setTimeout(later, wait);
		if (callNow) func.apply(context, args);
	};
};

const _enqueueUpdate = () => {
  Refresh.performReactRefresh()
}

export var registerExportsForReactRefresh = (moduleExports: ModExport, moduleID: string) => {
  if (moduleExports == null || typeof moduleExports !== 'object') {
    // Exit if we can't iterate over exports.
    // (This is important for legacy environments.)
    return;
  }

  for (const key in moduleExports) {
    const exportValue = moduleExports[key];
    if(Refresh.isLikelyComponentType(exportValue)) {
      const typeID = moduleID + ' exports ' + key;
      Refresh.register(exportValue, typeID);
    }
  }
};

export const enqueueUpdate = debounce(_enqueueUpdate, 30);