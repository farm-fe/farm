import { describe, expect, it, vi } from 'vitest';
import { HmrEngine } from '../src/server/hmr-engine.js';

describe('file changes during compilation', () => {
  it('defers native graph queries and preserves the pending update', async () => {
    let finish!: () => void;
    const compilation = new Promise<void>((resolve) => {
      finish = resolve;
    });
    const compiler = {
      compiling: true,
      waitForCompileFinish: vi.fn(() => compilation),
      hasModule: vi.fn(() => {
        if (compiler.compiling) {
          throw new Error('Native graph lock is held during compilation');
        }
        return true;
      })
    };
    const engine = new HmrEngine({ compiler } as any);
    const recompile = vi
      .spyOn(engine, 'recompileAndSendResult')
      .mockResolvedValue(undefined);
    const item = {
      path: '/commerce-nonexistent-pending-module.ts',
      type: 'updated' as const
    };
    const update = engine.hmrUpdate(item);
    const settled = update.then(
      () => ({ error: undefined }),
      (error: Error) => ({ error })
    );

    await Promise.resolve();
    expect(compiler.hasModule).not.toHaveBeenCalled();
    expect(recompile).not.toHaveBeenCalled();
    compiler.compiling = false;
    finish();
    expect((await settled).error).toBeUndefined();
    expect(compiler.hasModule).toHaveBeenCalledWith(item.path);
    expect(recompile).toHaveBeenCalledOnce();
    expect((engine as any)._updateQueue).toEqual([item]);
  });
});
