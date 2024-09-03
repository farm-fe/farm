import http, { createServer } from 'node:http';
import sirv from 'sirv';

interface RecordViewerServerOptions {
  clientPath: string;
  host?: string;
  port?: number;
  middleware: (
    req: http.IncomingMessage,
    res: http.ServerResponse,
    next: () => Promise<any>
  ) => void;
}
export function createRecordViewerServer(opts: RecordViewerServerOptions) {
  const { clientPath, host = 'localhost', port = 9527, middleware } = opts;
  const staticFilesServer = sirv(clientPath, {
    etag: true,
    single: true
  });
  const server = createServer();
  server.on('request', (req, res) => {
    middleware(req, res, () => {
      return Promise.resolve(staticFilesServer(req, res, () => {}));
    });
  });
  server.listen(port, host, () => {
    // console.log(`Farm Record Viewer run at http://${host}:${port}`);
  });
  return { server, host, port };
}
