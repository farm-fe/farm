import r1, { foo, unstable_batchedUpdates as batch, unstable_batchedUpdates1 } from './reexport';

const unstable_batchedUpdates = 123;
console.log({ unstable_batchedUpdates });

console.log({ r1, foo, batch, unstable_batchedUpdates1 });