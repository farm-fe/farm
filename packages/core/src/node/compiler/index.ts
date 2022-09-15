import {
  Compiler as BindingCompiler,
  JsUpdateResult,
} from '../../../binding/index';
import { normalizeUserCompilationConfig, UserConfig } from '../config';

export class Compiler {
  private _bindingCompiler: BindingCompiler;

  constructor(config: UserConfig) {
    this._bindingCompiler = new BindingCompiler(
      normalizeUserCompilationConfig(config)
    );
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
