export interface OwnedRuntimeServer {
  close(): Promise<void>;
}

/** A lifetime queue outside each server's native update queue. */
export class RuntimeServerOwner<T extends OwnedRuntimeServer> {
  private active?: T;
  private readonly owned = new Set<T>();
  private readonly cleanup = new WeakMap<T, Promise<void>>();
  private readonly controller = new AbortController();
  private queue: Promise<void> = Promise.resolve();
  private closed = false;
  private closing?: Promise<void>;

  get current(): T | undefined {
    return this.active;
  }

  /** Preparation owns partial allocations until it returns a fully usable candidate. */
  replace(prepare: (signal: AbortSignal) => Promise<T>): Promise<T> {
    const operation = this.queue.then(async () => {
      this.assertOpen();
      const candidate = await prepare(this.controller.signal);
      this.owned.add(candidate);
      if (this.closed) {
        await this.closeOne(candidate);
        throw new Error('[FARM_SERVER_OWNER] owner is closed');
      }
      const previous = this.active;
      this.active = candidate;
      // Publication is complete. Cleanup failure retains the new active instance
      // and remains observable to the caller and the final close operation.
      if (previous && previous !== candidate) await this.closeOne(previous);
      return candidate;
    });
    this.queue = operation.then(
      () => undefined,
      () => undefined
    );
    return operation;
  }

  close(): Promise<void> {
    if (!this.closing) {
      this.closed = true;
      this.controller.abort(new Error('[FARM_SERVER_OWNER] owner is closed'));
      this.closing = this.finishClose();
    }
    return this.closing;
  }

  private async finishClose(): Promise<void> {
    const started = new Set(
      [...this.owned].map((server) => this.closeOne(server))
    );
    const initial = Promise.allSettled([...started]);
    await this.queue;
    const remaining = [...this.owned]
      .map((server) => this.closeOne(server))
      .filter((promise) => !started.has(promise));
    const results = [
      ...(await initial),
      ...(await Promise.allSettled(remaining))
    ];
    this.active = undefined;
    const errors = results.flatMap((result) =>
      result.status === 'rejected' ? [result.reason] : []
    );
    if (errors.length)
      throw new AggregateError(
        errors,
        '[FARM_SERVER_OWNER] server cleanup failed'
      );
  }

  private closeOne(server: T): Promise<void> {
    let closing = this.cleanup.get(server);
    if (!closing) {
      closing = Promise.resolve()
        .then(() => server.close())
        .then(() => {
          this.owned.delete(server);
        });
      this.cleanup.set(server, closing);
    }
    return closing;
  }

  private assertOpen(): void {
    if (this.closed) throw new Error('[FARM_SERVER_OWNER] owner is closed');
  }
}
