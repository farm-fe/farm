import { Compiler } from '@farmfe/core';

const compiler = new Compiler({
  compilation: {
    input: {
      index: './index.ts',
    },
  },
});
compiler.compile();
