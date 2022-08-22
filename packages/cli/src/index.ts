import { Compiler } from '@farmfe/core';
// import path from 'path';

// compile config
// const configCompiler = new Compiler({
//   compilation: {
//     input: {
//       config: path.join(process.cwd(), "farm.config.ts")
//     }
//   }
// });

const compiler = new Compiler({
  compilation: {
    input: {
      index: './index.html',
    },
  },
});
compiler.compile();
