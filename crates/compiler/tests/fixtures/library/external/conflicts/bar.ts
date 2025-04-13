import { existsSync } from 'node:fs';

console.log('bar existsSync', existsSync('bar'));

export { existsSync } from 'node:fs';
export * from './zoo';