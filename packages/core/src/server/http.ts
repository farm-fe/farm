import type { Server } from 'node:http';
import { Logger } from '../utils/logger.js';

export async function httpServerStart(
  httpServer: Server,
  serverOptions: {
    port: number;
    strictPort: boolean | undefined;
    host: string | undefined;
    logger: Logger;
  }
): Promise<number> {
  let { port, strictPort, host, logger } = serverOptions;

  return new Promise((resolve, reject) => {
    const onError = (e: Error & { code?: string }) => {
      if (e.code === 'EADDRINUSE') {
        if (strictPort) {
          httpServer.removeListener('error', onError);
          reject(new Error(`Port ${port} is already in use`));
        } else {
          logger.info(`Port ${port} is in use, trying another one...`);
          httpServer.listen(++port, host);
        }
      } else {
        httpServer.removeListener('error', onError);
        reject(e);
      }
    };

    httpServer.on('error', onError);

    httpServer.listen(port, host, () => {
      httpServer.removeListener('error', onError);
      resolve(port);
    });
  });
}
