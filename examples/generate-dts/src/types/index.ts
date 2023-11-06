export type FixedPromise<T> = Promise<Awaited<T>>;

export type MaybePromise<T> = T | FixedPromise<T>;

export type IteratorResolve<T> = (args: IteratorResult<T>) => void;

export type Reject = (args: unknown) => void;
