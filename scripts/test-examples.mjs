import {
  buildCli,
  buildCoreCjs,
  buildExamples,
  buildRuntime,
  buildJsPlugins
} from './build.mjs';

console.log('Building CLI...');
await buildCli();
console.log('Building runtime...');
await buildRuntime();
console.log('Building core CJS...');
await buildCoreCjs();
console.log('Building JS plugins...');
await buildJsPlugins();
console.log('Building examples...');
await buildExamples();
