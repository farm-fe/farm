// @ts-nocheck
export { readFile as default } from 'node:fs';
export { foo } from '/external/foo';

export { unstable_batchedUpdates } from '/external/react-dom';
export { unstable_batchedUpdates as unstable_batchedUpdates1 } from './dep.cjs'