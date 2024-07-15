import type * as http from 'node:http';
import type { Http2SecureServer } from 'node:http2';

export type HttpServer = http.Server | Http2SecureServer;
