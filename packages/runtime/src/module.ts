/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
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

  d(to: any, to_k: string, val: any) {
    Object.defineProperty(to, to_k, {
      enumerable: true,
      get: function() {
        return val;
      }
    });
  }

  _m(to: any) {
    const key = '__esModule';
    if (to[key]) return;
    this.d(to, key, true);
  }

  // `export * from` helper
  _e(to: any, from: any) {
    const self = this;
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
          self.d(to, k, from[k]);
        }
    });
  
    return from;
  }

  // `export { xx } from` helper
  _(to: any, to_k: string, from: any, from_k: string) {
    this.d(to, to_k, from[from_k]);
  }
}
