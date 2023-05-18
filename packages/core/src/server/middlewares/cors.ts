import { DevServer } from '../index.js';
import cors from '@koa/cors';

export function corsPlugin(context: DevServer) {
  const { app, config } = context._context;
  if (!config.cors) return;
  app.use(cors(typeof config.cors === 'boolean' ? {} : config.cors));
}
