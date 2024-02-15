import { Middleware } from 'koa';
import { DevServer } from '../index.js';
import compress from 'koa-compress';

export function compression(devSeverContext: DevServer): Middleware {
  if (!devSeverContext.config.output) return;
  console.log('走进来这个插件了');

  return compress({
    // filter: (content_type) => {
    // console.log('compression filter', content_type);
    // return /text/i.test(content_type);
    // }
  });
}
