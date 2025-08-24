// Library 2 - Also re-exports from shared
export * from '../shared/common.js';
export { sharedFunction as shared } from '../shared/common.js';
export const uniqueToLib2 = () => 'Unique to Library 2';