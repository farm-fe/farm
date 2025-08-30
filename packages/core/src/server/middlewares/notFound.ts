import type { NextHandleFunction } from 'connect';

export function notFoundMiddleware(): NextHandleFunction {
  return function handle404Middleware(_, res) {
    res.statusCode = 404;
    res.end();
  };
}
