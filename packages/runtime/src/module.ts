/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
  meta: Record<string, any>;

  dispose?: () => void;

  constructor(id: string) {
    this.id = id;
    this.exports = {};
    this.meta = {
      env: {}
    };
  }

  onDispose(callback: () => void) {
    this.dispose = callback;
  }
}
