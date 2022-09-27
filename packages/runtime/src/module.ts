/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
  meta: Record<string, any>;
  initialized: boolean;

  constructor(id: string) {
    this.id = id;
    this.exports = {};
    this.meta = {};
    this.initialized = false;
  }
}
