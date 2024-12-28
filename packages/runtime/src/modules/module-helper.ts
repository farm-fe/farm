
import { getModuleSystem } from "../utils.js";

declare const __FARM_ENABLE_EXPORT_HELPER__: boolean;
declare const __FARM_ENABLE_EXPORT_ALL_HELPER__: boolean;
declare const __FARM_ENABLE_IMPORT_ALL_HELPER__: boolean;
declare const __FARM_IMPORT_EXPORT_FROM_HELPER__: boolean;
declare const __FARM_ENABLE_IMPORT_DEFAULT_HELPER__: boolean;

const moduleSystem = getModuleSystem();

const farmRequire: any = moduleSystem.r;

// These guards will be removed when the condition if false during compile time
if (__FARM_ENABLE_EXPORT_HELPER__) {
  farmRequire.o = (to: any, to_k: string, get: () => any) => {
    if (Object.prototype.hasOwnProperty.call(to, to_k)) {
      return;
    }
    Object.defineProperty(to, to_k, {
      enumerable: true,
      get
    });
  }
  
  // exports.xx = xx
  farmRequire.d = (to: any, to_k: string, val: any) => {
    farmRequire.o(to, to_k, function () {
      return val;
    });
  }
  
  // exports.__esModule
  farmRequire._m = (to: any) => {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, { value: true });
  }
}

if (__FARM_ENABLE_EXPORT_ALL_HELPER__) {
  // `export * from` helper
  farmRequire._e = (to: any, from: any) => {
    Object.keys(from).forEach(function (k) {
      if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
        Object.defineProperty(to, k, { value: from[k], enumerable: true, configurable: true });
      }
    });

    return from;
  }
}

if (__FARM_ENABLE_IMPORT_DEFAULT_HELPER__) {
  // `import xxx from` helper
  farmRequire.i = (obj: any) => {
    return obj && obj.__esModule ? obj : { default: obj };
  }
}

if (__FARM_ENABLE_IMPORT_ALL_HELPER__) {
  // `import * as xx` helper, copied from @swc/helper
  farmRequire._g = (nodeInterop: any) => {
    if (typeof WeakMap !== "function") return null;

    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();

    return (farmRequire._g = function (nodeInterop: any) {
      return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
  }

  // `import * as xx` helper, copied from @swc/helper
  farmRequire.w = (obj: any, nodeInterop: any) => {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return { default: obj };

    var cache = farmRequire._g(nodeInterop);

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
}

if (__FARM_IMPORT_EXPORT_FROM_HELPER__) {
  // `export { xx } from` helper
  farmRequire._ = (to: any, to_k: string, from: any, from_k: string) => {
    farmRequire.d(to, to_k, from[from_k || to_k]);
  }
}

if (__FARM_ENABLE_IMPORT_DEFAULT_HELPER__) {
  // minify x.default
  farmRequire.f = (v: any) => {
    if(typeof v.default !== 'undefined') {
      return v.default;
    }

    // compatible with `import default from "module"`
    return v;
  }
}