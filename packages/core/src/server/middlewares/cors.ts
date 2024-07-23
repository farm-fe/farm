import { default as koaCors } from '@koa/cors';
import { Middleware } from 'koa';
import { Server } from '../index.js';

export function cors(devSeverContext: Server): Middleware {
  const { config } = devSeverContext;
  if (!config.cors) return;
  return koaCors(typeof config.cors === 'boolean' ? {} : config.cors);
}

export const corsMiddleware = cors;
