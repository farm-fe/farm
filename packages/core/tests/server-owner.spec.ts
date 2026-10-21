import { describe, expect, it, vi } from 'vitest';
import { RuntimeServerOwner } from '../src/server/owner.js';

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const server = () => ({ close: vi.fn(async () => {}) });

describe('runtime server lifetime owner', () => {
  it('keeps the active server on preparation failure and closes it after a successful handoff', async () => {
    const owner = new RuntimeServerOwner<ReturnType<typeof server>>();
    const first = server(),
      next = server();
    await owner.replace(async () => first);
    await expect(
      owner.replace(async () => {
        throw new Error('candidate failed');
      })
    ).rejects.toThrow('candidate failed');
    expect(owner.current).toBe(first);
    expect(first.close).not.toHaveBeenCalled();
    await owner.replace(async () => next);
    expect(owner.current).toBe(next);
    expect(first.close).toHaveBeenCalledOnce();
    await owner.close();
    expect(next.close).toHaveBeenCalledOnce();
    expect(owner.current).toBeUndefined();
  });

  it('serializes candidates and leaves the current instance available during preparation', async () => {
    const owner = new RuntimeServerOwner<ReturnType<typeof server>>();
    const initial = server(),
      first = server(),
      second = server();
    await owner.replace(async () => initial);
    const gate = deferred();
    const prepare = vi.fn(async () => {
      await gate.promise;
      return first;
    });
    const later = vi.fn(async () => second);
    const one = owner.replace(prepare),
      two = owner.replace(later);
    await vi.waitFor(() => expect(prepare).toHaveBeenCalledOnce());
    expect(owner.current).toBe(initial);
    expect(later).not.toHaveBeenCalled();
    gate.resolve();
    await Promise.all([one, two]);
    expect(owner.current).toBe(second);
    expect(initial.close).toHaveBeenCalledOnce();
    expect(first.close).toHaveBeenCalledOnce();
    await owner.close();
  });

  it('cancels and waits for in-flight preparation, closes its late candidate and rejects queued work', async () => {
    const owner = new RuntimeServerOwner<ReturnType<typeof server>>();
    const initial = server(),
      candidate = server();
    await owner.replace(async () => initial);
    const gate = deferred();
    let receivedSignal: AbortSignal | undefined;
    const preparing = owner
      .replace(async (signal) => {
        receivedSignal = signal;
        await gate.promise;
        return candidate;
      })
      .catch((error) => error);
    const later = vi.fn(async () => server());
    const queued = owner.replace(later).catch((error) => error);
    await vi.waitFor(() => expect(receivedSignal).toBeDefined());
    let finished = false;
    const closing = owner.close();
    expect(owner.close()).toBe(closing);
    void closing.then(() => {
      finished = true;
    });
    expect(receivedSignal?.aborted).toBe(true);
    await Promise.resolve();
    expect(finished).toBe(false);
    await vi.waitFor(() => expect(initial.close).toHaveBeenCalledOnce());
    gate.resolve();
    expect(await preparing).toBeInstanceOf(Error);
    expect(await queued).toBeInstanceOf(Error);
    await closing;
    expect(later).not.toHaveBeenCalled();
    expect(candidate.close).toHaveBeenCalledOnce();
    expect(initial.close).toHaveBeenCalledOnce();
    expect(owner.current).toBeUndefined();
  });

  it('waits for old-instance cleanup during handoff and closes the new instance once', async () => {
    const owner = new RuntimeServerOwner<ReturnType<typeof server>>();
    const initial = server(),
      candidate = server(),
      gate = deferred();
    await owner.replace(async () => initial);
    initial.close.mockImplementationOnce(() => gate.promise);
    const replacing = owner.replace(async () => candidate);
    await vi.waitFor(() => expect(initial.close).toHaveBeenCalledOnce());
    expect(owner.current).toBe(candidate);
    let closed = false;
    const closing = owner.close().then(() => {
      closed = true;
    });
    await Promise.resolve();
    expect(closed).toBe(false);
    gate.resolve();
    await replacing;
    await closing;
    expect(initial.close).toHaveBeenCalledOnce();
    expect(candidate.close).toHaveBeenCalledOnce();
  });

  it('attempts current cleanup even if a retired instance failed to close', async () => {
    const owner = new RuntimeServerOwner<ReturnType<typeof server>>();
    const initial = server(),
      candidate = server();
    await owner.replace(async () => initial);
    initial.close.mockRejectedValueOnce(new Error('retired close failed'));
    await expect(owner.replace(async () => candidate)).rejects.toThrow(
      'retired close failed'
    );
    expect(owner.current).toBe(candidate);
    await expect(owner.close()).rejects.toThrow('server cleanup failed');
    expect(candidate.close).toHaveBeenCalledOnce();
    expect(owner.current).toBeUndefined();
  });
});
