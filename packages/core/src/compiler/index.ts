import type { Config, JsUpdateResult } from '../../binding/index.js';
import { Compiler as BindingCompiler } from '../../binding/index.js';

export class Compiler {
  private _bindingCompiler: BindingCompiler;

  config: Config;
  compiling = false;

  constructor(config: Config) {
    this.config = config;
    this._bindingCompiler = new BindingCompiler(this.config);
  }

  async compile() {
    if (this.compiling) {
      throw new Error('Already compiling');
    }
    this.compiling = true;
    await this._bindingCompiler.compile();
    this.compiling = false;
  }

  compileSync() {
    if (this.compiling) {
      throw new Error('Already compiling');
    }
    this.compiling = true;
    this._bindingCompiler.compileSync();
    this.compiling = false;
  }

  async update(paths: string[]): Promise<JsUpdateResult> {
    this.compiling = true;
    const res = await this._bindingCompiler.update(paths);
    this.compiling = false;
    return res;
  }

  updateSync(paths: string[]): JsUpdateResult {
    this.compiling = true;
    const res = this._bindingCompiler.updateSync(paths);
    this.compiling = false;
    return res;
  }

  hasModule(resolvedPath: string): boolean {
    return this._bindingCompiler.hasModule(resolvedPath);
  }
}
