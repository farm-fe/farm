import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import type { Config, JsUpdateResult } from '../../binding/index.js';
import { Compiler as BindingCompiler } from '../../binding/index.js';

export const VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX =
  'virtual:FARMFE_DYNAMIC_IMPORT:';

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

  resources(): Record<string, Buffer> {
    return this._bindingCompiler.resources();
  }

  resource(path: string): Buffer {
    return this._bindingCompiler.resource(path);
  }

  writeResourcesToDisk(): void {
    const resources = this.resources();
    const configOutputPath = this.config.config.output.path;
    const outputPath = path.isAbsolute(configOutputPath)
      ? configOutputPath
      : path.join(this.config.config.root, configOutputPath);

    for (const [name, resource] of Object.entries(resources)) {
      const filePath = path.join(outputPath, name);

      if (!existsSync(path.dirname(filePath))) {
        mkdirSync(path.dirname(filePath), { recursive: true });
      }

      writeFileSync(filePath, resource);
    }
  }

  removeOutputPathDir() {
    const outputPath = path.join(
      this.config.config.root,
      this.config.config.output.path
    );
    if (existsSync(outputPath)) {
      rmSync(outputPath, { recursive: true });
    }
  }

  resolvedModulePaths(root: string): string[] {
    return this._bindingCompiler
      .relativeModulePaths()
      .map((p) => this.transformModulePath(root, p));
  }

  transformModulePath(root: string, p: string): string {
    if (p.startsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX)) {
      return p.slice(VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX.length);
    }

    return path.join(root, p);
  }
}
