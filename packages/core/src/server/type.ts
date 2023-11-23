import http from 'node:http';
import http2 from 'node:http2';

export type Server = http.Server | http2.Http2SecureServer;
