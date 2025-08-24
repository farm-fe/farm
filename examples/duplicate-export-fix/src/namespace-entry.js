// Test for namespace collision and duplicate named exports
export * as utils from './shared/utils.js';
export * as helpers from './shared/utils.js';
export { formatDate, formatTime } from './shared/utils.js';
export * from './shared/utils.js';