import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { Logger, DefaultLogger } from '../utils/logger.js';
import { Compiler as BindingCompiler } from '../../binding/index.js';

import type { Config, JsUpdateResult } from '../../binding/index.js';

export const VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX =
  'virtual:FARMFE_DYNAMIC_IMPORT:';

/**
 * Cause the update process is async, we need to keep the update queue to make sure the update process is executed in order.
 * So the latter update process will not override the previous one if they are updating at the same time.
 */
export interface UpdateQueueItem {
  paths: string[];
  resolve: (res: JsUpdateResult) => void;
}

export class Compiler {
  private _bindingCompiler: BindingCompiler;
  private _updateQueue: UpdateQueueItem[] = [];
  private _onUpdateFinishQueue: (() => void)[] = [];

  public compiling = false;

  private logger: Logger;

  constructor(public config: Config) {
    this.logger = new DefaultLogger();
    this._bindingCompiler = new BindingCompiler(this.config);
  }

  async compile() {
    if (this.compiling) {
      this.logger.error('Already compiling');
    }
    this.compiling = true;
    if (process.env.FARM_PROFILE) {
      this._bindingCompiler.compileSync();
    } else {
      await this._bindingCompiler.compile();
    }
    this.compiling = false;
  }

  compileSync() {
    if (this.compiling) {
      this.logger.error('Already compiling');
    }
    this.compiling = true;
    this._bindingCompiler.compileSync();
    this.compiling = false;
  }

  async update(
    paths: string[],
    sync = false,
    ignoreCompilingCheck = false
  ): Promise<JsUpdateResult> {
    let resolve: (res: JsUpdateResult) => void;

    const promise = new Promise<JsUpdateResult>((r) => {
      resolve = r;
    });

    // if there is already a update process, we need to wait for it to finish
    if (this.compiling && !ignoreCompilingCheck) {
      this._updateQueue.push({ paths, resolve });
      return promise;
    }

    this.compiling = true;
    try {
      const res = await this._bindingCompiler.update(
        paths,
        async () => {
          const next = this._updateQueue.shift();

          if (next) {
            await this.update(next.paths, true, true).then(next.resolve);
          } else {
            this.compiling = false;
            this._onUpdateFinishQueue.forEach((cb) => cb());
            // clear update finish queue
            this._onUpdateFinishQueue = [];
          }
        },
        sync
      );

      return res as JsUpdateResult;
    } catch (e) {
      this.compiling = false;
      throw e;
    }
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
      if (process.env.NODE_ENV === 'test') {
        console.log('Writing', name, 'to disk');
      }

      const filePath = path.join(outputPath, name);

      if (!existsSync(path.dirname(filePath))) {
        mkdirSync(path.dirname(filePath), { recursive: true });
      }

      writeFileSync(filePath, resource);
    }
  }

  removeOutputPathDir() {
    const outputPath = this.outputPath();
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

  onUpdateFinish(cb: () => void) {
    this._onUpdateFinishQueue.push(cb);
  }

  outputPath() {
    const { output, root } = this.config.config;
    const configOutputPath = output.path;
    const outputPath = path.isAbsolute(configOutputPath)
      ? configOutputPath
      : path.join(root, configOutputPath);
    return outputPath;
  }
}
