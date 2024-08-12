export function publicPathMiddleware(app: any) {
  return function handlePublicPathMiddleware(req: any, res: any, next: any) {
    next();
  };
}
