import { buildCli, buildJsPlugins, buildExamples } from './build.mjs';

console.log('Building CLI...');
await buildCli();
console.log('Building JS plugins...');
await buildJsPlugins();

// read all directories under examples and run `npm run build` in each directory
await buildExamples();
