import { expect, it, vi } from 'vitest';
import { Compiler } from '../src/compiler/index.js';
import { HmrEngine } from '../src/server/hmr-engine.js';

const native = vi.hoisted(() => ({
  callbacks: [] as Array<() => Promise<void>>,
  paths: [] as Array<Array<[string, string]>>,
  results: [] as Array<() => void>,
  failures: [] as Array<(error: Error) => void>,
  failP: false,
  entered: undefined as (() => void) | undefined
}));

vi.mock('../binding/index.js', () => ({
  Compiler: class {
    hasModule() {
      return true;
    }
    async compile() {
      native.paths.push([['external.compile', 'compile']]);
      native.callbacks.push(async () => {});
      await new Promise<void>((resolve, reject) => {
        native.results.push(resolve);
        native.failures.push(reject);
      });
    }
    async update(
      paths: Array<[string, string]>,
      completion: () => Promise<void>
    ) {
      native.paths.push(paths);
      native.callbacks.push(completion);
      const result = new Promise<void>((resolve, reject) => {
        native.results.push(resolve);
        native.failures.push(reject);
      });
      native.entered?.();
      if (paths.some(([path]) => path === '/owner-P.virtual') && native.failP) {
        native.failP = false;
        throw new Error('P attempted once and failed');
      }
      // P's native result is immediate if ownership is correctly restored;
      // its resource callback remains separate and is completed explicitly below.
      if (!paths.some(([path]) => path === '/owner-P.virtual')) await result;
      return {
        added: [],
        changed: [],
        removed: [],
        immutableModules: '{}',
        mutableModules: '{}',
        boundaries: {}
      };
    }
  },
  Resolver: class {},
  JsFileWatcher: class {},
  bindingPath: ''
}));
vi.mock('../src/utils/index.js', () => ({
  bold: (value: string) => value,
  cyan: (value: string) => value,
  green: (value: string) => value,
  normalizePath: (value: string) => value.replaceAll('\\', '/'),
  cleanUrl: (value: string) => value.split('?')[0]
}));

it.each(
  [
    ['public', 'update-success'],
    ['watcher', 'update-success'],
    ['public', 'update-failure'],
    ['watcher', 'update-failure'],
    ['public', 'compile-success'],
    ['watcher', 'compile-success'],
    ['watcher', 'update-success-redundant'],
    ['watcher', 'update-success-p-failure'],
    ['watcher', 'update-success-redundant-p-failure']
  ].flatMap(([admission, external]) =>
    [true, false].map((retained) => [admission, external, retained] as const)
  )
)(
  'retains an HMR owner after A callback consumption (%s, external B %s, callback retained %s)',
  async (admission, external, retained) => {
    native.callbacks = [];
    native.paths = [];
    native.results = [];
    native.failures = [];
    const pFails = external.endsWith('p-failure');
    const redundant = external.includes('redundant');
    native.failP = pFails;
    let entered!: () => void;
    const aEntered = new Promise<void>((resolve) => {
      entered = resolve;
    });
    native.entered = entered;
    // Use the real JS Compiler readiness and callback queues, mocking only native.
    const compiler = new Compiler({ root: '/', compilation: {} } as any);
    let unblockCompletion!: () => void;
    let completionEntered!: () => void;
    const completionBlocked = new Promise<void>((resolve) => {
      completionEntered = resolve;
    });
    const completionRelease = new Promise<void>((resolve) => {
      unblockCompletion = resolve;
    });
    const blockListener = async () => {
      completionEntered();
      await completionRelease;
    };
    if (retained) compiler.onUpdateFinish(blockListener);
    const engine = new HmrEngine({
      compiler,
      config: { server: { hmr: {} } },
      logger: { info: vi.fn(), error: vi.fn(), formatTime: () => '0ms' },
      ws: { wss: { clients: new Set() } }
    } as any);
    // Observe completion of actual submit calls without changing their semantics.
    // This is a deterministic continuation barrier, not a sleep-based assertion.
    const submissions = vi.spyOn(engine as any, 'submitHmrUpdate');
    const a = engine.hmrUpdate(
      { path: '/owner-A.virtual', type: 'updated' },
      true
    );
    await aEntered;
    if (!retained) compiler.onUpdateFinish(blockListener);
    // Both A signals settle, then its async success listener holds the HMR
    // continuation while external B claims compiler readiness.
    const aCompletion = native.callbacks[0]();
    native.results[0]();
    await completionBlocked;
    expect(compiler.compiling).toBe(false);
    expect((compiler as any)._onUpdateFinishQueue).toHaveLength(
      retained ? 1 : 0
    );
    const p = { path: '/owner-P.virtual', type: 'updated' as const };
    let prepared!: () => void;
    let releasePreparation!: () => void;
    const preparationDone = new Promise<void>((resolve) => {
      prepared = resolve;
    });
    const preparationRelease = new Promise<void>((resolve) => {
      releasePreparation = resolve;
    });
    const originalPrepare = (engine as any).prepareHmrUpdate.bind(engine);
    vi.spyOn(engine as any, 'prepareHmrUpdate').mockImplementation(
      async (...args) => {
        await originalPrepare(...args);
        prepared();
        await preparationRelease;
      }
    );
    const pendingAdmission =
      admission === 'public'
        ? engine.hmrUpdate(p, true)
        : engine.queueWatchUpdate(p);
    await preparationDone;
    expect((engine as any)._updateQueue).toContainEqual(p);
    // External B has its own readiness/result; it is not an HMR submission.
    // Attach rejection handling immediately; a rejected B is not a P attempt.
    const failure = new Error('external B failed');
    const b = (
      external === 'compile-success'
        ? compiler.compile()
        : compiler.update([{ path: '/owner-B.virtual', type: 'updated' }])
    ).then(
      () => undefined,
      (error) => error
    );
    expect(compiler.compiling).toBe(true);
    releasePreparation();
    unblockCompletion();
    await aCompletion;
    await a;
    let enteredP!: () => void;
    const pEntered = new Promise<void>((resolve) => {
      enteredP = resolve;
    });
    native.entered = () => {
      if (native.paths.at(-1)?.some(([path]) => path === p.path)) enteredP();
    };
    if (external.startsWith('update-success')) {
      native.callbacks[1]();
      if (redundant) native.callbacks[1]();
    }
    if (external === 'update-failure') native.failures[1](failure);
    else native.results[1]();
    await pEntered;
    if (!pFails) native.callbacks[2]();
    const bResult = await b;
    expect(bResult).toBe(external === 'update-failure' ? failure : undefined);
    await pendingAdmission;
    await Promise.all(submissions.mock.results.map((result) => result.value));
    expect(
      native.paths.flat().filter(([path]) => path === p.path)
    ).toHaveLength(1);
    await compiler.waitForCompileFinish();
    // Desired invariant, deliberately not softened to characterize a failure as
    // success. Compare public arm on base to classify inheritance vs regression.
    expect(
      native.paths.flat().filter(([path]) => path === p.path)
    ).toHaveLength(1);
    if (pFails) {
      // A failed P is retained, but readiness alone did not retry it.
      // Only this subsequent valid trigger may attempt it a second time.
      const retryEntered = new Promise<void>((resolve) => {
        native.entered = resolve;
      });
      const retry = engine.hmrUpdate(p, true);
      await retryEntered;
      await native.callbacks[3]();
      await retry;
      await compiler.waitForCompileFinish();
      expect(
        native.paths.flat().filter(([path]) => path === p.path)
      ).toHaveLength(2);
    }
  }
);
