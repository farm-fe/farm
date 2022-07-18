import {
  Compiler as BindingCompiler,
  JsUpdateResult,
} from '../../../binding/index';
import { normalizeUserCompilationConfig, UserConfig } from '../config';

export class Compiler {
  bindingCompiler: BindingCompiler;

  constructor(config: UserConfig) {
    this.bindingCompiler = new BindingCompiler(
      normalizeUserCompilationConfig(config)
    );
  }

  async compile() {
    await this.bindingCompiler.compile();
    console.log('after rust compile');
  }

  compileSync() {
    return this.bindingCompiler.compileSync();
  }

  async update(paths: string[]): Promise<JsUpdateResult> {
    return this.bindingCompiler.update(paths);
  }

  updateSync(paths: string[]): JsUpdateResult {
    return this.bindingCompiler.updateSync(paths);
  }
}
