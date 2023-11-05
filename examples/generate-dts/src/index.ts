import type { FixedPromise } from '@/types';

export function isPromise<A>(args: unknown): args is FixedPromise<A> {
  if (args instanceof Promise) {
    return true;
  }

  if (
    args !== null &&
    typeof args === 'object' &&
    typeof (args as any).then === 'function' &&
    typeof (args as any).catch === 'function'
  ) {
    return true;
  }

  return false;
}
