import type Connect from 'connect';

export function hmrPingMiddleware(): Connect.NextHandleFunction {
  return function handleHMRPingMiddleware(req, res, next) {
    if (req.headers['accept'] === 'text/x-farm-ping') {
      res.writeHead(204).end();
    } else {
      next();
    }
  };
}
