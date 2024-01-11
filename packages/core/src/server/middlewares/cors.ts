import { Middleware } from 'koa';
import { DevServer } from '../index.js';
import { default as koaCors } from '@koa/cors';

export function cors(devSeverContext: DevServer): Middleware {
  const { config } = devSeverContext;
  if (!config.cors) return;
  return koaCors(typeof config.cors === 'boolean' ? {} : config.cors);
}
