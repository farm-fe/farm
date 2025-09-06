import { existsSync, rmSync } from 'node:fs';
import path from 'node:path';

import { Compiler as BindingCompiler } from '../../binding/index.js';

import type {
  ResolvedCompilation,
  ResolvedUserConfig,
  Resource
} from '../index.js';
import type {
  CompilerUpdateItem,
  JsApiMetadata,
  JsUpdateResult
} from '../types/binding.js';
import { cleanUrl } from '../utils/index.js';

export const VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX =
  '.farm_dynamic_import_virtual_module' as const;

/**
 * Cause the update process is async, we need to keep the update queue to make sure the update process is executed in order.
 * So the latter update process will not override the previous one if they are updating at the same time.
 */
export interface UpdateQueueItem {
  paths: CompilerUpdateItem[];
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

  constructor(public config: ResolvedUserConfig) {
    this._bindingCompiler = new BindingCompiler({
      config: config.compilation,
      jsPlugins: config.jsPlugins,
      rustPlugins: config.rustPlugins
    });
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
    try {
      if (process.env.FARM_PROFILE) {
        this._bindingCompiler.compileSync();
      } else {
        await this._bindingCompiler.compile();
      }
    } catch (e) {
      // error thrown from rust compiler do not have js stack trace
      e.stack = '';
      throw e;
    } finally {
      this.compiling = false;
      this._resolveCompileFinishPromise();
    }
  }

  compileSync() {
    this.checkCompiling();
    this._createCompileFinishPromise();
    this.compiling = true;
    this._bindingCompiler.compileSync();
    this.compiling = false;
    this._resolveCompileFinishPromise();
  }

  async update(
    paths: CompilerUpdateItem[],
    sync = false,
    ignoreCompilingCheck = false,
    generateUpdateResource = true
  ): Promise<JsUpdateResult> {
    let resolve: (res: JsUpdateResult) => void;

    const promise = new Promise<JsUpdateResult>((r) => {
      resolve = r;
    });

    // if there is already a update process, we need to wait for it to finish
    if (this.isCompilingAndCheckIgnored(ignoreCompilingCheck)) {
      this._updateQueue.push({ paths, resolve });
      return promise;
    }

    this.compiling = true;

    try {
      const res = await this._bindingCompiler.update(
        paths.map((item) => [item.path, item.type]),
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

  private isCompilingAndCheckIgnored(ignoreCompilingCheck: boolean): boolean {
    return this.compiling && !ignoreCompilingCheck;
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
    return (
      this._bindingCompiler.resource(path) ||
      this._bindingCompiler.resource(cleanUrl(path))
    );
  }

  resourcesMap(): Record<string, Resource> {
    return this._bindingCompiler.resourcesMap() as Record<string, Resource>;
  }

  /**
   * Writes the compiled resources to disk and calls the write resources hook.
   */
  writeResourcesToDisk(): void {
    this._bindingCompiler.writeResourcesToDisk();
  }

  callWriteResourcesHook() {
    for (const jsPlugin of this.config.jsPlugins) {
      jsPlugin.writeResources?.executor?.({
        resourcesMap: this._bindingCompiler.resourcesMap() as Record<
          string,
          Resource
        >,
        config: this.config.compilation
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
      this.config.logger.error('Already compiling', {
        exit: true
      });
    }
  }

  private getOutputPath(): string {
    const { output, root } = this.config.compilation;
    const configOutputPath = output.path;
    const outputPath = path.isAbsolute(configOutputPath)
      ? configOutputPath
      : path.join(root, configOutputPath);
    return outputPath;
  }

  invalidateModule(moduleId: string) {
    this._bindingCompiler.invalidateModule(moduleId);
  }

  /**
   * write cache
   * @param name cache name
   * @param data data to be cached
   * @param options cache options
   */
  writeCache<V>(name: string, data: V, options?: JsApiMetadata | null): void {
    this._bindingCompiler.writeCache(name, JSON.stringify(data), options);
  }

  /**
   * read cache
   * @param name cache name
   * @param options cache options
   * @returns cached data, `undefined` if not exists
   */
  readCache<V>(name: string, options?: JsApiMetadata | null): V | undefined {
    const data = this._bindingCompiler.readCache(name, options);
    return data ? JSON.parse(data) : undefined;
  }
}

export function createCompiler(resolvedUserConfig: ResolvedUserConfig) {
  return new Compiler(resolvedUserConfig);
}

export function createInlineCompiler(
  config: ResolvedUserConfig,
  options: ResolvedCompilation = {}
) {
  return new Compiler({
    ...config,
    compilation: { ...config.compilation, ...options }
  });
}
