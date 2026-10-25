// @ts-ignore
import Refresh from 'react-refresh';

type ModExport = Record<string, any>;

type RefreshFamily = { current: unknown };
let registrationTransaction: { previous: Map<RefreshFamily, unknown> } | undefined;
let refreshRevision = 0;

/** Register through the Rust-injected entry point so a batch can retain its old families. */
export function register(type: any, id: string): void {
  refreshRevision++;
  if (registrationTransaction) {
    const capture = (value: any, key: string) => {
      const family = Refresh.getFamilyByID(key) as RefreshFamily | undefined;
      if (family && !registrationTransaction!.previous.has(family)) {
        registrationTransaction!.previous.set(family, family.current);
      }
      if (value !== null && typeof value === 'object') {
        if (value.$$typeof === Symbol.for('react.memo')) capture(value.type, key + '$type');
        else if (value.$$typeof === Symbol.for('react.forward_ref')) capture(value.render, key + '$render');
      }
    };
    capture(type, id);
  }
  Refresh.register(type, id);
}

/** Registry boundary only; module and React DOM recovery remain the caller's responsibility. */
export async function beginRefreshTransaction() {
  if (registrationTransaction) throw new Error('React Refresh transaction already active');
  do { await flushRefresh(); }
  while (registrations.size || refreshTask || registrationErrors.length || refreshFailures.length);
  if (registrationTransaction) throw new Error('React Refresh transaction already active');
  const transaction = { previous: new Map<RefreshFamily, unknown>() };
  registrationTransaction = transaction;
  let closing = false;
  let validatedRevision: number | undefined;
  let settledRevision: number | undefined;
  const errors: unknown[] = [];
  const assertOpen = () => {
    if (closing || registrationTransaction !== transaction) throw new Error('React Refresh transaction closed');
    closing = true;
  };
  const isSettled = () => !closing && registrationTransaction === transaction && settledRevision === refreshRevision &&
    !registrations.size && !refreshTask && !registrationErrors.length && !refreshFailures.length;
  const settle = async (): Promise<void> => {
    assertOpen();
    try {
      do {
        try { await flushRefresh(); } catch (error) { errors.push(error); }
      } while (registrations.size || refreshTask || registrationErrors.length || refreshFailures.length);
      settledRevision = refreshRevision;
    }
    finally { closing = false; }
  };
  const validate = async (): Promise<void> => {
    validatedRevision = undefined;
    do { await settle(); } while (!isSettled());
    if (errors.length) throw new AggregateError(errors, `React Refresh transaction failed: ${errors.map(String).join('; ')}`);
    validatedRevision = settledRevision;
  };
  const restore = (): unknown[] => {
    if (closing || registrationTransaction !== transaction) throw new Error('React Refresh transaction closed');
    if (!isSettled()) throw new Error('React Refresh transaction must be settled before restoration');
    for (const [family, previous] of transaction.previous) family.current = previous;
    registrationTransaction = undefined;
    return [...errors];
  };
  const finalize = (): void => {
    if (closing || registrationTransaction !== transaction) throw new Error('React Refresh transaction closed');
    if (validatedRevision !== refreshRevision || !isSettled() || errors.length) {
      throw new Error('React Refresh transaction must be validated after its latest work');
    }
    registrationTransaction = undefined;
  };
  return {
    settle,
    isSettled,
    restore,
    validate,
    finalize,
    async commit(): Promise<void> {
      await validate(); finalize();
    },
    async rollback(): Promise<unknown[]> {
      do { await settle(); } while (!isSettled());
      return restore();
    }
  };
}

// @ts-ignore
export async function validateRefreshBoundaryAndEnqueueUpdate(prevExports: ModExport, nextExports: ModExport) {
  const ignoredExports = []
  if (
    predicateOnExport(
      ignoredExports,
      prevExports,
      (key) => key in nextExports,
    ) !== true
  ) {
    return 'Could not Fast Refresh (export removed)'
  }
  if (
    predicateOnExport(
      ignoredExports,
      nextExports,
      (key) => key in prevExports,
    ) !== true
  ) {
    return 'Could not Fast Refresh (new export)'
  }

  let hasExports = false
  const allExportsAreComponentsOrUnchanged = predicateOnExport(
    ignoredExports,
    nextExports,
    (key, value) => {
      hasExports = true
      if (Refresh.isLikelyComponentType(value)) return true
      return prevExports[key] === nextExports[key]
    },
  )

  if (hasExports && allExportsAreComponentsOrUnchanged === true) {
    await enqueueUpdate()
  } else {
    return `Could not Fast Refresh ("${allExportsAreComponentsOrUnchanged}" export is incompatible).`
  }
}

function predicateOnExport(ignoredExports: string[], moduleExports: ModExport, predicate: (key: string, value: any) => boolean): string | true {
  for (const key in moduleExports) {
    if (key === '__esModule') continue
    if (ignoredExports.includes(key)) continue
    if (!predicate(key, moduleExports[key])) return key
  }
  return true
}

let refreshTimer: ReturnType<typeof setTimeout> | undefined;
let refreshTask: { promise: Promise<void>; resolve(): void; reject(error: unknown): void } | undefined;
const refreshFailures: { task: NonNullable<typeof refreshTask>; error: unknown }[] = [];
const registrations = new Set<Promise<void>>();
const registrationErrors: unknown[] = [];

/** Preserve export initialization ordering while exposing the real registration boundary. */
export function scheduleRegistration(register: () => void): void {
  refreshRevision++;
  let finish!: () => void;
  const completion = new Promise<void>(resolve => { finish = resolve; });
  registrations.add(completion);
  setTimeout(() => {
    try { register(); }
    catch (error) { registrationErrors.push(error); }
    finally { registrations.delete(completion); finish(); }
  });
}

/** Coalesced callers settle only after the actual React Refresh invocation. */
export function enqueueUpdate(): Promise<void> {
  refreshRevision++;
  if (!refreshTask) {
    let resolve!: () => void;
    let reject!: (error: unknown) => void;
    const promise = new Promise<void>((success, failure) => { resolve = success; reject = failure; });
    refreshTask = { promise, resolve, reject };
  }
  const task = refreshTask;
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => {
    refreshTask = undefined;
    refreshTimer = undefined;
    try { Refresh.performReactRefresh(); task.resolve(); }
    catch (error) { refreshFailures.push({ task, error }); task.reject(error); }
  }, 30);
  return task.promise;
}

/** Await all injected registration work and its refresh, rather than a guessed delay. */
export async function flushRefresh(): Promise<void> {
  do {
    while (registrations.size) await Promise.all([...registrations]);
    if (registrationErrors.length) throw new AggregateError(registrationErrors.splice(0), 'React Refresh registration failed');
    if (refreshFailures.length) throw new AggregateError(refreshFailures.splice(0).map(failure => failure.error), 'React Refresh failed');
    const pending = refreshTask;
    if (pending) {
      try { await pending.promise; }
      catch (error) {
        const index = refreshFailures.findIndex(failure => failure.task === pending);
        if (index !== -1) refreshFailures.splice(index, 1);
        throw error;
      }
    }
  } while (registrations.size || refreshTask || refreshFailures.length);
}

export var registerExportsForReactRefresh = (moduleExports: ModExport, moduleID: string) => {
  if (moduleExports == null || typeof moduleExports !== 'object') {
    // Exit if we can't iterate over exports.
    // (This is important for legacy environments.)
    return;
  }

  for (const key in moduleExports) {
    const exportValue = moduleExports[key];
    if(Refresh.isLikelyComponentType(exportValue)) {
      const typeID = moduleID + ' exports ' + key;
      register(exportValue, typeID);
    }
  }
};
