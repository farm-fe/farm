import { Compiler } from '../../compiler/index.js';
import { ResolvedUserConfig } from '../../config/types.js';
import { HttpServer } from '../index.js';

export function publicMiddleware(
  server: HttpServer,
  compiler: Compiler,
  publicPath: string,
  config: ResolvedUserConfig
) {
  return async function handlerPublicMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {};
}
