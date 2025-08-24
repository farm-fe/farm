// Test for duplicate exports with complex re-export patterns
// This should generate duplicate exports that get deduplicated
export * from './lib/library1.js';
export * from './lib/library2.js';
export { shared as sharedFromLib1 } from './lib/library1.js';
export { shared as sharedFromLib2 } from './lib/library2.js';