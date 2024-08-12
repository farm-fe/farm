export function publicPathMiddleware(app: any) {
  return function handlePublicPathMiddleware(req: any, res: any, next: any) {
    // auto redirect to public path
    // e.g:
    // if (req.url.startsWith("/public/")) {
    //   res.redirect(301, req.url.replace("/public/", "/"));
    // }
    next();
  };
}
