/* eslint-disable @typescript-eslint/no-explicit-any */
export class Module {
  id: string;
  exports: any;
  meta: Record<string, any>;

  constructor(id: string) {
    this.id = id;
    this.exports = {};
    this.meta = {};
  }
}
