import { cleanUrl } from '../../utils/url.js';

export function publicPathMiddleware(app: any) {
  return function handlePublicPathMiddleware(req: any, res: any, next: any) {
    // auto redirect to public path
    // e.g:      res.writeHead(302, {
    //  Location: base + url.slice(pathname.length),
    // })
    // console.log(req.url);
    const url = req.url!;
    const pathname = cleanUrl(url);

    next();
  };
}
