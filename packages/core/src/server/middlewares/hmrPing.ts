export function hmrPingMiddleware() {
  return function handleHMRPingMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    if (req.headers['accept'] === 'text/x-farm-ping') {
      res.writeHead(204).end();
    } else {
      next();
    }
  };
}
