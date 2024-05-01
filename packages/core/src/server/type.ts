import type http from 'node:http';
import type http2 from 'node:http2';

export type Server = http.Server | http2.Http2SecureServer;
