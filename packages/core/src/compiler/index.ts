import { existsSync, realpathSync, rmSync } from 'node:fs';
import path from 'node:path';

import { Compiler as BindingCompiler } from '../../binding/index.js';
import type { NativeDocsSnapshot } from '../docs-native.js';

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

interface PendingUpdate extends UpdateQueueItem {
  sync: boolean;
  generateUpdateResource: boolean;
  reject: (error: unknown) => void;
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

/** Runtime ownership is intentionally separate from serializable configuration. */
export interface CompilerRuntimeOptions {
  docsSnapshot?: NativeDocsSnapshot;
}

export class Compiler {
  private _bindingCompiler: BindingCompiler;
  private _updateQueue: PendingUpdate[] = [];
  private _onUpdateFinishQueue: (() => void | Promise<void>)[] = [];

  public compiling = false;
  private _compileFinishPromise: Promise<void> | null = null;
  private _resolveCompileFinish: (() => void) | null = null;

  constructor(
    public config: ResolvedUserConfig,
    runtime: CompilerRuntimeOptions = {}
  ) {
    this._bindingCompiler = new BindingCompiler({
      config: config.compilation,
      jsPlugins: config.jsPlugins,
      rustPlugins: config.rustPlugins,
      ...(runtime.docsSnapshot ? { docsSnapshot: runtime.docsSnapshot } : {})
    });
  }

  inputObservations(): unknown[] {
    return JSON.parse(this._bindingCompiler.inputObservations()) as unknown[];
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
    } catch (e: any) {
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
    // Initialize with a no-op that is immediately overwritten by the synchronous
    // Promise executor; avoids TS2454 with strictNullChecks
    let resolve: (res: JsUpdateResult) => void = () => {};
    let reject: (error: unknown) => void = () => {};

    const promise = new Promise<JsUpdateResult>((r, j) => {
      resolve = r;
      reject = j;
    });

    // if there is already a update process, we need to wait for it to finish
    if (this.isCompilingAndCheckIgnored(ignoreCompilingCheck)) {
      this._updateQueue.push({
        paths,
        sync,
        generateUpdateResource,
        resolve,
        reject
      });
      return promise;
    }

    // Reuse the readiness promise while draining queued updates. The binding
    // returns an update result before asynchronous resource generation ends.
    if (!this._compileFinishPromise) {
      this._createCompileFinishPromise();
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
              next.sync,
              true,
              next.generateUpdateResource
            ).then(next.resolve, next.reject);
          } else {
            this.compiling = false;
            this._resolveCompileFinishPromise();
            while (this._onUpdateFinishQueue.length) {
              if (this.compiling) {
                break;
              }
              const cb = this._onUpdateFinishQueue.shift();
              if (cb) {
                await cb();
              }
            }
          }
        },
        sync,
        generateUpdateResource
      );

      return res as JsUpdateResult;
    } catch (e: any) {
      // A failed native update may never invoke its completion callback. Settle
      // every waiting caller instead of leaving requests pending indefinitely.
      for (const queued of this._updateQueue.splice(0)) {
        queued.reject(e);
      }
      this.compiling = false;
      this._resolveCompileFinishPromise();
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
    // Match the resolver's physical file identity, retaining query variants.
    // Virtual IDs and symlinks:false keep their original identity.
    let resolved = idOrResolvedPath;
    if (this.config.compilation?.resolve?.symlinks !== false) {
      const filename = cleanUrl(idOrResolvedPath);
      const root =
        this.config.compilation?.root ?? this.config.root ?? process.cwd();
      const absolute = path.resolve(root, filename);
      if (existsSync(absolute)) {
        resolved =
          realpathSync(absolute) + idOrResolvedPath.slice(filename.length);
      }
    }
    return this._bindingCompiler.getParentFiles(resolved);
  }

  resources(): Record<string, Buffer> {
    return this._bindingCompiler.resources();
  }

  resource(path: string): Buffer | null {
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
    for (const jsPlugin of this.config.jsPlugins ?? []) {
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
    while (this.compiling && this._compileFinishPromise) {
      await this._compileFinishPromise;
    }
  }

  /** Drain native cache writes after callers stop admitting compilation. */
  async flushCache(): Promise<void> {
    await this.waitForCompileFinish();
    await this._bindingCompiler.flushCache();
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
      this.config.logger?.error('Already compiling', {
        exit: true
      });
    }
  }

  private getOutputPath(): string {
    const compilation = this.config.compilation ?? {};
    const { output, root } = compilation;
    const configOutputPath = output?.path ?? 'dist';
    const outputPath = path.isAbsolute(configOutputPath)
      ? configOutputPath
      : path.join(root ?? process.cwd(), configOutputPath);
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
  writeMetadata<V>(
    name: string,
    data: V,
    options?: JsApiMetadata | null
  ): void {
    this._bindingCompiler.writeMetadata(name, JSON.stringify(data), options);
  }

  /**
   * read cache
   * @param name cache name
   * @param options cache options
   * @returns cached data, `undefined` if not exists
   */
  readMetadata<V>(name: string, options?: JsApiMetadata | null): V | undefined {
    const data = this._bindingCompiler.readMetadata(name, options);
    return data ? JSON.parse(data) : undefined;
  }

  readMetadataByScope<V>(scope: string): V[] {
    return this._bindingCompiler
      .readMetadataByScope(scope)
      .map((item) => JSON.parse(item));
  }
}

export function createCompiler(
  resolvedUserConfig: ResolvedUserConfig,
  runtime: CompilerRuntimeOptions = {}
) {
  return new Compiler(resolvedUserConfig, runtime);
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
