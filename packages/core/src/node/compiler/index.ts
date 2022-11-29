import {
  Compiler as BindingCompiler,
  Config,
  JsUpdateResult,
} from '../../../binding/index';
import { normalizeUserCompilationConfig, UserConfig } from '../config';

export class Compiler {
  private _bindingCompiler: BindingCompiler;
  config: Config;

  constructor(config: UserConfig) {
    this.config = normalizeUserCompilationConfig(config);

    this._bindingCompiler = new BindingCompiler(this.config);
  }

  async compile() {
    await this._bindingCompiler.compile();
    console.log('after rust compile');
  }

  compileSync() {
    return this._bindingCompiler.compileSync();
  }

  async update(paths: string[]): Promise<JsUpdateResult> {
    return this._bindingCompiler.update(paths);
  }

  updateSync(paths: string[]): JsUpdateResult {
    return this._bindingCompiler.updateSync(paths);
  }

  resources(): Record<string, number[]> {
    return this._bindingCompiler.resources();
  }
}
