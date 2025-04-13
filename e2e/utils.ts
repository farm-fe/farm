import fs from 'node:fs/promises';
import { isAbsolute, join } from 'node:path';
export type SimpleUnwrapArray<T> = T extends ReadonlyArray<infer P> ? P : T;

export function logger(msg: any, { title = 'FARM INFO', color = 'green' } = {}) {
  const COLOR_CODE = [
    'black',
    'red',
    'green',
    'yellow',
    'blue',
    'magenta',
    'cyan',
    'white',
  ].indexOf(color);
  if (COLOR_CODE >= 0) {
    const TITLE_STR = title ? `\x1b[4${COLOR_CODE};30m ${title} \x1b[0m ` : '';
    console.log(`${TITLE_STR}\x1b[3${COLOR_CODE}m${msg}\x1b[;0m`);
  } else {
    console.log(title ? `${title} ${msg}` : msg);
  }
}

export interface Deferred<T = any> {
  resolve: (result: T) => void;
  reject: (reason: any) => void;
  promise: Promise<T>;
}

export const createDeferred = <T = any>(silent?: boolean) => {
  const deferred = {} as Deferred<T>;

  deferred.promise = new Promise<T>((resolve, reject) => {
    deferred.resolve = resolve;
    deferred.reject = reject;
  });

  if (silent) {
    deferred.promise.catch(() => {});
  }

  return deferred;
};


export const concurrentify = <F extends (...args: any) => Promise<any>>(maxConcurrent: number, fn: F) => {
  const queue = [] as {
    deferred: Deferred;
    args: any;
    ctx: any;
  }[];

  let concurrent = 0;

  function next() {
    concurrent -= 1;
    if (queue.length > 0) {
      const { ctx, deferred, args } = queue.shift()!;
      try {
        newFn.apply(ctx, args).then(deferred.resolve, deferred.reject);
      } catch (e) {
        deferred.reject(e);
      }
    }
  }

  function newFn(this: any) {
    const ctx = this;
    const args = arguments as any;

    if (concurrent >= maxConcurrent) {
      const deferred = createDeferred();
      queue.push({
        deferred,
        ctx,
        args,
      });
      return deferred.promise;
    }

    concurrent += 1;

    return fn.apply(ctx, args).finally(next);
  }

  return newFn as F;
};

export const concurrentMap = <
  Arr extends readonly unknown[],
  F extends (item: SimpleUnwrapArray<Arr>, index: number, arr: Arr) => Promise<any>,
>(arr: Arr, maxConcurrent: number, cb: F) => arr.map(
  concurrentify(maxConcurrent, cb) as any,
) as ReturnType<F>[];


export async function editFile(_filename: string, matched: string | RegExp, to: string): Promise<undefined | (() => Promise<void>)> {
  const filename = isAbsolute(_filename) ? _filename : join(process.cwd(), _filename);
  const content = await fs.readFile(filename, 'utf-8')

  let newContent = content.replaceAll(matched, to);

  if(content.length !== newContent.length || content !== newContent) {
    await fs.writeFile(filename, newContent);

    return async () => {
      await fs.writeFile(filename, content)
    }
  }
}