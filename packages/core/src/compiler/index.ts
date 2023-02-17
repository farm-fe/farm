import { existsSync, mkdirSync } from 'fs';
import fs from 'fs/promises';
import path from 'path';

import type { Config, JsUpdateResult } from '../../binding/index.js';
import { Compiler as BindingCompiler } from '../../binding/index.js';
// import { normalizeUserCompilationConfig } from '../config/index.js';

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

  resources(): Record<string, number[]> {
    return this._bindingCompiler.resources();
  }

  async writeResourcesToDisk(): Promise<void> {
    const resources = this.resources();
    const promises = [];

    for (const [name, resource] of Object.entries(resources)) {
      const filePath = path.join(this.config.config.output.path, name);

      if (!existsSync(path.dirname(filePath))) {
        mkdirSync(path.dirname(filePath), { recursive: true });
      }

      promises.push(fs.writeFile(filePath, Buffer.from(resource)));
    }

    await Promise.all(promises);
  }
}
