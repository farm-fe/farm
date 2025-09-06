import getEtag from 'etag';
import { IncomingMessage, OutgoingHttpHeaders, ServerResponse } from 'http';
import mime from 'mime';
import { extname } from 'path/posix';

export interface SendOptions {
  etag?: string;
  cacheControl?: string;
  headers?: OutgoingHttpHeaders;
}

export function send(
  req: IncomingMessage,
  res: ServerResponse,
  content: string | Buffer,
  url: string,
  options: SendOptions
): void {
  const {
    etag = getEtag(content, { weak: true }),
    cacheControl = 'no-cache',
    headers
  } = options;

  if (res.writableEnded) {
    return;
  }

  if (req.headers['if-none-match'] === etag) {
    res.statusCode = 304;
    res.end();
    return;
  }

  res.setHeader('Content-Type', mime.getType(extname(url)));
  res.setHeader('Cache-Control', cacheControl);
  res.setHeader('Etag', etag);

  if (headers) {
    for (const name in headers) {
      res.setHeader(name, headers[name]);
    }
  }

  res.statusCode = 200;
  res.end(content);
  return;
}
