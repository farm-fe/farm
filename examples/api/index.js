import { build } from '@farmfe/core'

await build({ compilation: { input: { entry: 'index.ts' } } });

console.log('build success');

