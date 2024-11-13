/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
  // initialize promise if this module is a async module
  initializer: Promise<any>;
  resource_pot: string;
  meta: Record<string, any>;
  require: (id: string) => any;

  constructor(id: string, require: (id: string) => any) {
    this.id = id;
    this.exports = {};
    this.meta = {
      env: {}
    };
    this.require = require;
  }

  o(to: any, to_k: string, get: () => any) {
    Object.defineProperty(to, to_k, {
      enumerable: true,
      get
    });
  }

  // exports.xx = xx
  d(to: any, to_k: string, val: any) {
    this.o(to, to_k, function () {
      return val;
    });
  }

  // exports.__esModule
  _m(to: any) {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, { value: true });
  }

  // `export * from` helper
  _e(to: any, from: any) {
    Object.keys(from).forEach(function (k) {
      if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
        Object.defineProperty(to, k, { value: from[k], enumerable: true, configurable: true });
      }
    });

    return from;
  }

  // `import xxx from` helper
  i(obj: any) {
    return obj && obj.__esModule ? obj : { default: obj };
  }
  // `import * as xx` helper, copied from @swc/helper
  _g(nodeInterop: any) {
    if (typeof WeakMap !== "function") return null;

    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();

    return (this._g = function (nodeInterop) {
      return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
  }
  // `import * as xx` helper, copied from @swc/helper
  w(obj: any, nodeInterop: any) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return { default: obj };

    var cache = this._g(nodeInterop);

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
  _(to: any, to_k: string, from: any, from_k: string) {
    this.d(to, to_k, from[from_k || to_k]);
  }

  // revert imports/exports minify helper
  p(to: Record<string, string>, val: any) {
    for (const key of Object.keys(val)) {
      const newKey = to[key];
      if (newKey && !Object.prototype.hasOwnProperty.call(val, newKey)) {
        this.d(val, newKey, val[key]);
      }
    }
  }

  // minify x.default
  f(v: any) {
    if(typeof v.default !== 'undefined') {
      return v.default;
    }

    // compatible `import default from "module"`
    return v;
  }
}
