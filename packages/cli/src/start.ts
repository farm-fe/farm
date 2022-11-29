import { Compiler, DevServer } from '@farmfe/core';

export async function start(): Promise<void> {
  const compiler = new Compiler({
    compilation: {
      input: {
        index: './index.html',
      },
    },
  });

  const devServer = new DevServer(compiler, {
    writeToDisk: true,
  });
  devServer.listen();

  // compiler.compile();
}
