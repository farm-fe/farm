import type { Compiler } from '../compiler/index.js';
import type { CompilerUpdateItem, JsUpdateResult } from '../types/binding.js';

export type RuntimeCompiler = Pick<
  Compiler,
  'compile' | 'update' | 'waitForCompileFinish' | 'resources' | 'flushCache'
>;

/** Serial native compilation and an independently owned successful resource generation. */
export class ServerRuntime {
  private queue: Promise<void> = Promise.resolve();
  private closed = false;
  private closing?: Promise<void>;
  private readonly ownedCompilers = new Set<RuntimeCompiler>();
  private readonly retiringCacheDrains = new Map<
    RuntimeCompiler,
    Promise<void>
  >();
  private readonly cacheDrainErrors: unknown[] = [];
  private published: Record<string, Buffer> = Object.create(null);

  constructor(private activeCompiler: RuntimeCompiler) {
    this.ownedCompilers.add(activeCompiler);
  }

  get compiler(): RuntimeCompiler {
    return this.activeCompiler;
  }

  /** Prepare an independent compiler while retaining the last successful generation. */
  replaceCompiler(compiler: RuntimeCompiler): Promise<void> {
    return this.enqueue(async () => {
      // A previously failed candidate may be repaired and reused. Its old
      // drain must release ownership before this compilation can start.
      await this.retiringCacheDrains.get(compiler);
      this.assertOpen();
      this.ownedCompilers.add(compiler);
      try {
        await compiler.compile();
        await compiler.waitForCompileFinish();
        this.assertOpen();
        const candidate = this.copyResources(compiler);
        const previous = this.activeCompiler;
        // No await separates the compiler identity and resource generation commit.
        this.activeCompiler = compiler;
        this.published = candidate;
        if (previous !== compiler) this.retireCompiler(previous);
      } catch (error) {
        if (compiler !== this.activeCompiler) this.retireCompiler(compiler);
        throw error;
      }
    });
  }

  compile(): Promise<void> {
    return this.enqueue(async () => {
      await this.compiler.compile();
      await this.compiler.waitForCompileFinish();
      this.publish();
    });
  }

  update(paths: CompilerUpdateItem[]): Promise<JsUpdateResult> {
    const batch = paths.map((item) => ({ ...item }));
    return this.enqueue(async () => {
      const result = await this.compiler.update(batch, true);
      await this.compiler.waitForCompileFinish();
      this.publish();
      return result;
    });
  }

  resource(name: string): Buffer | undefined {
    const bytes = this.published[name];
    return bytes === undefined ? undefined : Buffer.from(bytes);
  }

  resources(): Record<string, Buffer> {
    return Object.fromEntries(
      Object.entries(this.published).map(([name, bytes]) => [
        name,
        Buffer.from(bytes)
      ])
    );
  }

  close(): Promise<void> {
    this.closed = true;
    return (this.closing ??= this.queue.then(async () => {
      const results = await Promise.allSettled(
        [...this.ownedCompilers].map(
          (compiler) =>
            this.retiringCacheDrains.get(compiler) ?? compiler.flushCache()
        )
      );
      this.ownedCompilers.clear();
      const failures = results.filter(
        (result): result is PromiseRejectedResult =>
          result.status === 'rejected'
      );
      const errors = [
        ...this.cacheDrainErrors,
        ...failures.map((result) => result.reason)
      ];
      if (errors.length)
        throw new AggregateError(errors, 'Native cache drain failed');
    }));
  }

  private retireCompiler(compiler: RuntimeCompiler): void {
    if (this.retiringCacheDrains.has(compiler)) return;
    const drain = Promise.resolve()
      .then(() => compiler.flushCache())
      .catch((error) => {
        this.cacheDrainErrors.push(error);
      })
      .finally(() => {
        this.ownedCompilers.delete(compiler);
        this.retiringCacheDrains.delete(compiler);
      });
    this.retiringCacheDrains.set(compiler, drain);
  }

  private assertOpen(): void {
    if (this.closed) throw new Error('[FARM_SERVER_RUNTIME] runtime is closed');
  }

  private publish(): void {
    this.assertOpen();
    this.published = this.copyResources(this.compiler);
  }

  private copyResources(compiler: RuntimeCompiler): Record<string, Buffer> {
    const candidate: Record<string, Buffer> = Object.create(null);
    for (const [name, bytes] of Object.entries(compiler.resources())) {
      candidate[name] = Buffer.from(bytes);
    }
    return candidate;
  }

  private enqueue<T>(operation: () => Promise<T>): Promise<T> {
    const task = this.queue.then(() => {
      this.assertOpen();
      return operation();
    });
    // Keep the sequencing tail fulfilled so a failed batch cannot poison repair.
    this.queue = task.then(
      () => undefined,
      () => undefined
    );
    return task;
  }
}
