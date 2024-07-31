import type * as http from 'node:http';
// import type * as net from 'node:net';
// import connect from 'connect';
import httpProxy from 'http-proxy';

export interface ProxyOptions extends httpProxy.ServerOptions {
  rewrite?: (path: string) => string;
  configure?: (proxy: httpProxy, options: ProxyOptions) => void;
  bypass?: (
    req: http.IncomingMessage,
    res: http.ServerResponse,
    options: ProxyOptions
  ) => void | null | undefined | false | string;
  rewriteWsOrigin?: boolean | undefined;
}
