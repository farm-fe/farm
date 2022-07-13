import {
  Compiler as BindingCompiler,
  JsUpdateResult,
} from '../../binding/index';
import { normalizeUserConfig, UserConfig } from './config';

export class Compiler {
  bindingCompiler: BindingCompiler;

  constructor(config: UserConfig) {
    this.bindingCompiler = new BindingCompiler(normalizeUserConfig(config));
  }

  async compile() {
    return this.bindingCompiler.compile();
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
