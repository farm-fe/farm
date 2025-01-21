import type { ModuleSystem } from "../module-system.js";

declare const __FARM_ENABLE_EXPORT_HELPER__: boolean;
declare const __FARM_ENABLE_EXPORT_ALL_HELPER__: boolean;
declare const __FARM_ENABLE_IMPORT_ALL_HELPER__: boolean;
declare const __FARM_IMPORT_EXPORT_FROM_HELPER__: boolean;
declare const __FARM_ENABLE_IMPORT_DEFAULT_HELPER__: boolean;

export function initModuleSystem(ms: ModuleSystem) {
  const farmRequire: any = ms.r;

  // These guards will be removed when the condition if false during compile time
  if (__FARM_ENABLE_EXPORT_HELPER__) {
    farmRequire.o = exportByDefineProperty;
    // exports.xx = xx
    farmRequire.d = defineExport;
    // exports.__esModule
    farmRequire._m = defineExportEsModule;
  }

  if (__FARM_ENABLE_EXPORT_ALL_HELPER__) {
    // `export * from` helper
    farmRequire._e = defineExportStar;
  }

  if (__FARM_ENABLE_IMPORT_DEFAULT_HELPER__) {
    // `import xxx from` helper
    farmRequire.i = interopRequireDefault;
  }

  if (__FARM_ENABLE_IMPORT_ALL_HELPER__) {
    // `import * as xx` helper, copied from @swc/helper
    farmRequire._g = getRequireWildcardCache;
    // `import * as xx` helper, copied from @swc/helper
    farmRequire.w = interopRequireWildcard;
  }

  if (__FARM_IMPORT_EXPORT_FROM_HELPER__) {
    farmRequire._ = defineExportFrom;
  }

  if (__FARM_ENABLE_IMPORT_DEFAULT_HELPER__) {
    farmRequire.f = importDefault;
  }
}

function exportByDefineProperty(to: any, to_k: string, get: () => any) {
  if (Object.prototype.hasOwnProperty.call(to, to_k)) {
    return;
  }
  Object.defineProperty(to, to_k, {
    enumerable: true,
    get
  });
}

function defineExport(to: any, to_k: string, val: any) {
  exportByDefineProperty(to, to_k, function () {
    return val;
  });
}

// exports.__esModule
export function defineExportEsModule(to: any) {
  const key = '__esModule';
  if (to[key]) return;
  Object.defineProperty(to, key, { value: true });
}

// `export * from` helper
export function defineExportStar(to: any, from: any) {
  Object.keys(from).forEach(function (k) {
    if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
      Object.defineProperty(to, k, { value: from[k], enumerable: true, configurable: true });
    }
  });

  return from;
}

// `import xxx from` helper
export function interopRequireDefault(obj: any) {
  return obj && obj.__esModule ? obj : { default: obj };
}

function getRequireWildcardCache(nodeInterop: any) {
  if (typeof WeakMap !== "function") return null;

  var cacheBabelInterop = new WeakMap();
  var cacheNodeInterop = new WeakMap();

  // @ts-ignore ignore type check
  return (getRequireWildcardCache = function (nodeInterop: any) {
    return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
  })(nodeInterop);
}

// `import * as xx` helper, copied from @swc/helper
export function interopRequireWildcard(obj: any, nodeInterop: any) {
  if (!nodeInterop && obj && obj.__esModule) return obj;
  if (obj === null || typeof obj !== "object" && typeof obj !== "function") return { default: obj };

  var cache = getRequireWildcardCache(nodeInterop);

  if (cache && cache.has(obj)) return cache.get(obj);

  var newObj: any = { __proto__: null };
  var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;

  for (var key in obj) {
      if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
          var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
          if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
          else newObj[key] = obj[key];
      }
  }

  newObj.default = obj;

  if (cache) cache.set(obj, newObj);

  return newObj;
}

// `export { xx } from` helper
function defineExportFrom(to: any, to_k: string, from: any, from_k: string) {
  defineExport(to, to_k, from[from_k || to_k]);
}

// minify x.default
function importDefault(v: any) {
  if(typeof v.default !== 'undefined') {
    return v.default;
  }

  // compatible with `import default from "module"`
  return v;
}
