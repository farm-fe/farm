export const createDeferred = (silent) => {
  const deferred = {};

  deferred.promise = new Promise<T>((resolve, reject) => {
    deferred.resolve = resolve;
    deferred.reject = reject;
  });

  if (silent) {
    deferred.promise.catch(() => {});
  }

  return deferred;
};
export const concurrentify = (maxConcurrent, fn) => {
  const queue = [];

  let concurrent = 0;

  function next() {
    concurrent -= 1;
    if (queue.length > 0) {
      const { ctx, deferred, args } = queue.shift();
      try {
        newFn.apply(ctx, args).then(deferred.resolve, deferred.reject);
      } catch (e) {
        deferred.reject(e);
      }
    }
  }

  function newFn(context) {
    const ctx = context;
    const args = arguments;

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

  return newFn;
};

export const concurrentMap = (arr, maxConcurrent, cb) => arr.map(
  concurrentify(maxConcurrent, cb),
);


