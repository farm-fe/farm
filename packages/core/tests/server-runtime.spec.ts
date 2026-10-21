import { describe, expect, it, vi } from 'vitest';
import { ServerRuntime } from '../src/server/runtime.js';

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function fixture() {
  let resources: Record<string, Buffer> = {
    'index.html': Buffer.from('initial')
  };
  const compiler = {
    compile: vi.fn(async () => {}),
    update: vi.fn(
      async (_paths: unknown, _sync: boolean): Promise<any> => ({
        changed: ['entry.js']
      })
    ),
    waitForCompileFinish: vi.fn(async () => {}),
    flushCache: vi.fn(async () => {}),
    resources: vi.fn(() => resources)
  };
  return {
    compiler,
    runtime: new ServerRuntime(compiler),
    setResources: (next: Record<string, Buffer>) => {
      resources = next;
    }
  };
}

describe('server runtime successful resource generations', () => {
  it('waits for native cache drain on close and shares its completion', async () => {
    const { runtime, compiler } = fixture();
    const gate = deferred();
    compiler.flushCache.mockImplementationOnce(() => gate.promise);
    await runtime.compile();
    let settled = false;
    const closing = runtime.close().then(() => {
      settled = true;
    });
    await vi.waitFor(() => expect(compiler.flushCache).toHaveBeenCalledOnce());
    expect(settled).toBe(false);
    const second = runtime.close();
    gate.resolve();
    await Promise.all([closing, second]);
    expect(compiler.flushCache).toHaveBeenCalledOnce();
  });

  it('drains failed candidates and retired compilers even when another drain rejects', async () => {
    const previous = fixture();
    const failed = fixture();
    const next = fixture();
    await previous.runtime.compile();
    failed.compiler.compile.mockRejectedValueOnce(
      new Error('candidate failed')
    );
    await expect(
      previous.runtime.replaceCompiler(failed.compiler)
    ).rejects.toThrow('candidate failed');
    await previous.runtime.replaceCompiler(next.compiler);
    next.compiler.flushCache.mockRejectedValueOnce(
      new Error('writer panicked')
    );
    await expect(previous.runtime.close()).rejects.toThrow(
      'Native cache drain failed'
    );
    for (const compiler of [previous.compiler, failed.compiler, next.compiler])
      expect(compiler.flushCache).toHaveBeenCalledOnce();
    await expect(previous.runtime.close()).rejects.toThrow(
      'Native cache drain failed'
    );
  });

  it('publishes a replacement compiler only after success and directs queued updates to it', async () => {
    const previous = fixture();
    const candidate = fixture();
    candidate.setResources({ 'index.html': Buffer.from('replacement') });
    await previous.runtime.compile();
    const gate = deferred();
    candidate.compiler.compile.mockImplementationOnce(() => gate.promise);
    const replacement = previous.runtime.replaceCompiler(candidate.compiler);
    const update = previous.runtime.update([
      { path: '/next.js', type: 'updated' }
    ]);
    await vi.waitFor(() =>
      expect(candidate.compiler.compile).toHaveBeenCalledOnce()
    );
    expect(previous.runtime.compiler).toBe(previous.compiler);
    expect(previous.runtime.resource('index.html')?.toString()).toBe('initial');
    expect(candidate.compiler.update).not.toHaveBeenCalled();
    gate.resolve();
    await replacement;
    await update;
    expect(previous.runtime.compiler).toBe(candidate.compiler);
    expect(previous.runtime.resource('index.html')?.toString()).toBe(
      'replacement'
    );
    expect(previous.compiler.update).not.toHaveBeenCalled();
    expect(candidate.compiler.update).toHaveBeenCalledOnce();
  });

  it('retains the active compiler and resources when replacement fails, then accepts repair', async () => {
    const previous = fixture();
    const candidate = fixture();
    await previous.runtime.compile();
    candidate.setResources({ 'index.html': Buffer.from('unpublished') });
    candidate.compiler.compile.mockRejectedValueOnce(
      new Error('candidate configuration failed')
    );
    await expect(
      previous.runtime.replaceCompiler(candidate.compiler)
    ).rejects.toThrow('candidate configuration failed');
    expect(previous.runtime.compiler).toBe(previous.compiler);
    expect(previous.runtime.resource('index.html')?.toString()).toBe('initial');
    await previous.runtime.update([]);
    expect(previous.compiler.update).toHaveBeenCalledOnce();
    await previous.runtime.replaceCompiler(candidate.compiler);
    expect(previous.runtime.compiler).toBe(candidate.compiler);
    expect(previous.runtime.resource('index.html')?.toString()).toBe(
      'unpublished'
    );
  });

  it('waits for replacement during close without publishing or admitting queued updates', async () => {
    const previous = fixture();
    const candidate = fixture();
    await previous.runtime.compile();
    const gate = deferred();
    candidate.compiler.compile.mockImplementationOnce(() => gate.promise);
    const replacement = previous.runtime
      .replaceCompiler(candidate.compiler)
      .catch((error) => error);
    await vi.waitFor(() =>
      expect(candidate.compiler.compile).toHaveBeenCalledOnce()
    );
    const queued = previous.runtime.update([]).catch((error) => error);
    let closed = false;
    const closing = previous.runtime.close().then(() => {
      closed = true;
    });
    await Promise.resolve();
    expect(closed).toBe(false);
    gate.resolve();
    expect(await replacement).toBeInstanceOf(Error);
    expect(await queued).toBeInstanceOf(Error);
    await closing;
    expect(previous.runtime.compiler).toBe(previous.compiler);
    expect(previous.runtime.resource('index.html')?.toString()).toBe('initial');
    expect(candidate.compiler.update).not.toHaveBeenCalled();
  });

  it('keeps the previous generation while native resources are partially changed', async () => {
    const { compiler, runtime, setResources } = fixture();
    await runtime.compile();
    const gate = deferred();
    compiler.update.mockImplementationOnce(async () => {
      setResources({
        'index.html': Buffer.from('next'),
        'next.js': Buffer.from('next script')
      });
      await gate.promise;
      return { changed: ['entry.js'] };
    });
    const update = runtime.update([{ path: '/entry.js', type: 'updated' }]);
    await vi.waitFor(() => expect(compiler.update).toHaveBeenCalledOnce());
    expect(runtime.resource('index.html')?.toString()).toBe('initial');
    expect(runtime.resource('next.js')).toBeUndefined();
    gate.resolve();
    await update;
    expect(compiler.update).toHaveBeenCalledWith(
      [{ path: '/entry.js', type: 'updated' }],
      true
    );
    expect(runtime.resource('index.html')?.toString()).toBe('next');
    expect(runtime.resource('next.js')?.toString()).toBe('next script');
  });

  it('preserves the old generation on failure and accepts the next repair', async () => {
    const { compiler, runtime, setResources } = fixture();
    await runtime.compile();
    compiler.update.mockImplementationOnce(async () => {
      setResources({ 'index.html': Buffer.from('partial broken') });
      throw new Error('late native failure');
    });
    await expect(runtime.update([])).rejects.toThrow('late native failure');
    expect(runtime.resource('index.html')?.toString()).toBe('initial');
    setResources({ 'index.html': Buffer.from('repaired') });
    await runtime.update([]);
    expect(runtime.resource('index.html')?.toString()).toBe('repaired');
  });

  it('serializes updates without starting the next native batch early', async () => {
    const { compiler, runtime } = fixture();
    const gate = deferred();
    compiler.update.mockImplementationOnce(async () => {
      await gate.promise;
      return { changed: ['first.js'] };
    });
    const first = runtime.update([{ path: '/first.js', type: 'updated' }]);
    const second = runtime.update([{ path: '/second.js', type: 'added' }]);
    await vi.waitFor(() => expect(compiler.update).toHaveBeenCalledTimes(1));
    gate.resolve();
    await Promise.all([first, second]);
    expect(compiler.update.mock.calls.map((call) => call[0])).toEqual([
      [{ path: '/first.js', type: 'updated' }],
      [{ path: '/second.js', type: 'added' }]
    ]);
  });

  it('owns buffer copies on both publication and reads', async () => {
    const { compiler, runtime } = fixture();
    await runtime.compile();
    compiler.resources()['index.html'].fill(0);
    const resource = runtime.resource('index.html');
    if (!resource) throw new Error('Expected the compiled HTML resource');
    resource.fill(1);
    runtime.resources()['index.html'].fill(2);
    expect(runtime.resource('index.html')?.toString()).toBe('initial');
  });

  it('waits for in-flight work and rejects queued work when closing', async () => {
    const { compiler, runtime } = fixture();
    await runtime.compile();
    const gate = deferred();
    compiler.update.mockImplementationOnce(async () => {
      await gate.promise;
      return {};
    });
    const first = runtime.update([]);
    const queued = runtime.update([]);
    const settled = Promise.allSettled([first, queued]);
    await vi.waitFor(() => expect(compiler.update).toHaveBeenCalledOnce());
    let closed = false;
    const closing = runtime.close().then(() => {
      closed = true;
    });
    await Promise.resolve();
    expect(closed).toBe(false);
    gate.resolve();
    await closing;
    const results = await settled;
    expect(results.every((result) => result.status === 'rejected')).toBe(true);
    expect(compiler.update).toHaveBeenCalledOnce();
    await expect(runtime.update([])).rejects.toThrow('closed');
    await runtime.close();
    expect(runtime.resource('index.html')?.toString()).toBe('initial');
  });
});
