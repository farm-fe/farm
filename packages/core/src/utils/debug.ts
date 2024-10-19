import debug from 'debug';

const DEBUG = process.env.DEBUG;

interface DebuggerOptions {
  onlyWhenFocused?: boolean | string;
}

export type FarmDebugScope = `farm:${string}`;

export function createDebugger(
  namespace: FarmDebugScope,
  options: DebuggerOptions = {}
): debug.Debugger['log'] | undefined {
  const log = debug(namespace);
  const { onlyWhenFocused } = options;

  let enabled = log.enabled;

  if (enabled && onlyWhenFocused) {
    const ns =
      typeof onlyWhenFocused === 'string' ? onlyWhenFocused : namespace;

    enabled = !!DEBUG?.includes(ns);
  }

  if (enabled) {
    return (...args: [string, ...any[]]) => {
      log(...args);
    };
  }
}
