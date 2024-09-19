import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { Compiler as BindingCompiler } from '../../binding/index.js';

import type { Resource } from '../index.js';
import type { Config, JsUpdateResult } from '../types/binding.js';
import { type ILogger, Logger } from '../utils/logger.js';

export const VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX =
  '.farm_dynamic_import_virtual_module';

/**
 * Cause the update process is async, we need to keep the update queue to make sure the update process is executed in order.
 * So the latter update process will not override the previous one if they are updating at the same time.
 */
export interface UpdateQueueItem {
  paths: string[];
  resolve: (res: JsUpdateResult) => void;
}

export interface TracedModuleGraph {
  root: string;
  modules: Array<{
    id: string;
    contentHash: string;
    packageName: string;
    packageVersion: string;
  }>;
  edges: Record<string, string[]>;
  reverseEdges: Record<string, string[]>;
}

export class Compiler {
  private _bindingCompiler: BindingCompiler;
  private _updateQueue: UpdateQueueItem[] = [];
  private _onUpdateFinishQueue: (() => void | Promise<void>)[] = [];

  public compiling = false;
  private _compileFinishPromise: Promise<void> | null = null;
  private _resolveCompileFinish: (() => void) | null = null;
  private _isInitialCompile = true;

  constructor(
    public config: Config,
    private logger: ILogger = new Logger()
  ) {
    this._bindingCompiler = new BindingCompiler(this.config);
  }

  async traceDependencies() {
    return this._bindingCompiler.traceDependencies();
  }

  async traceModuleGraph(): Promise<TracedModuleGraph> {
    return this._bindingCompiler.traceModuleGraph() as TracedModuleGraph;
  }

  async compile() {
    this.checkCompiling();
    this._createCompileFinishPromise();
    this.compiling = true;
    if (process.env.FARM_PROFILE) {
      this._bindingCompiler.compileSync();
    } else {
      await this._bindingCompiler.compile();
    }
    this.compiling = false;
    this._resolveCompileFinishPromise();
    if (this._isInitialCompile) {
      this._isInitialCompile = false;
    }
  }

  compileSync() {
    this.checkCompiling();
    this._createCompileFinishPromise();
    this.compiling = true;
    this._bindingCompiler.compileSync();
    this.compiling = false;
    this._resolveCompileFinishPromise();
    if (this._isInitialCompile) {
      this._isInitialCompile = false;
    }
  }

  async update(
    paths: string[],
    sync = false,
    ignoreCompilingCheck = false,
    generateUpdateResource = true
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
            await this.update(
              next.paths,
              true,
              true,
              generateUpdateResource
            ).then(next.resolve);
          } else {
            this.compiling = false;
            while (this._onUpdateFinishQueue.length) {
              if (this.compiling) {
                break;
              }
              const cb = this._onUpdateFinishQueue.shift();
              await cb();
            }
          }
        },
        sync,
        generateUpdateResource
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

  getParentFiles(idOrResolvedPath: string): string[] {
    return this._bindingCompiler.getParentFiles(idOrResolvedPath);
  }

  resources(): Record<string, Buffer> {
    return this._bindingCompiler.resources();
  }

  resource(path: string): Buffer {
    return this._bindingCompiler.resource(path);
  }

  resourcesMap(): Record<string, Resource> {
    return this._bindingCompiler.resourcesMap() as Record<string, Resource>;
  }

  writeResourcesToDisk(): void {
    const resources = this.resources();
    const outputPath = this.getOutputPath();

    for (const [name, resource] of Object.entries(resources)) {
      const filePath = path.join(outputPath, name.split(/[?#]/)[0]);

      if (!existsSync(path.dirname(filePath))) {
        mkdirSync(path.dirname(filePath), { recursive: true });
      }

      writeFileSync(filePath, resource);
    }

    this.callWriteResourcesHook();
  }

  callWriteResourcesHook() {
    for (const jsPlugin of this.config.jsPlugins) {
      jsPlugin.writeResources?.executor?.({
        resourcesMap: this._bindingCompiler.resourcesMap() as Record<
          string,
          Resource
        >,
        config: this.config.config
      });
    }
  }

  removeOutputPathDir() {
    const outputPath = this.outputPath();
    if (existsSync(outputPath)) {
      rmSync(outputPath, { recursive: true });
    }
  }

  resolvedWatchPaths(): string[] {
    return this._bindingCompiler.watchModules();
  }

  resolvedModulePaths(root: string): string[] {
    return this._bindingCompiler
      .relativeModulePaths()
      .map((p) => this.transformModulePath(root, p));
  }

  transformModulePath(root: string, p: string): string {
    if (p.endsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX)) {
      p = p.slice(0, -VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX.length);
    }

    return path.isAbsolute(p) ? p : path.join(root, p.split('?')[0]);
  }

  onUpdateFinish(cb: () => void) {
    this._onUpdateFinishQueue.push(cb);
  }

  outputPath() {
    return this.getOutputPath();
  }

  addExtraWatchFile(root: string, paths: string[]) {
    this._bindingCompiler.addWatchFiles(root, paths);
  }

  stats() {
    return this._bindingCompiler.stats();
  }

  // wait for the compiler to finish compiling
  async waitForInitialCompileFinish() {
    if (this._isInitialCompile) {
      await this.waitForCompileFinish();
    }
  }

  async waitForCompileFinish() {
    if (this.compiling && this._compileFinishPromise) {
      await this._compileFinishPromise;
    }
  }

  private _createCompileFinishPromise() {
    this._compileFinishPromise = new Promise<void>((resolve) => {
      this._resolveCompileFinish = resolve;
    });
  }

  private _resolveCompileFinishPromise() {
    if (this._resolveCompileFinish) {
      this._resolveCompileFinish();
      this._compileFinishPromise = null;
      this._resolveCompileFinish = null;
    }
  }

  private checkCompiling() {
    if (this.compiling) {
      this.logger.error('Already compiling', {
        exit: true
      });
    }
  }

  private getOutputPath(): string {
    const { output, root } = this.config.config;
    const configOutputPath = output.path;
    const outputPath = path.isAbsolute(configOutputPath)
      ? configOutputPath
      : path.join(root, configOutputPath);
    return outputPath;
  }
}
