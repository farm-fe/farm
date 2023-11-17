/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
  resource_pot: string;
  meta: Record<string, any>;
  require: (id: string) => any;

  dispose?: () => void;

  constructor(id: string, require: (id: string) => any) {
    this.id = id;
    this.exports = {};
    this.meta = {
      env: {}
    };
    this.require = require;
  }

  onDispose(callback: () => void) {
    this.dispose = callback;
  }
}
