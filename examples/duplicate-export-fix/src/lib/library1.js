// Library 1 - Re-exports from shared
export * from '../shared/common.js';
export { sharedFunction as shared } from '../shared/common.js';
export const uniqueToLib1 = () => 'Unique to Library 1';