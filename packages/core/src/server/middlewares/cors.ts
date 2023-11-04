import { DevServer } from '../index.js';
import cors from '@koa/cors';

export function corsPlugin(devSeverContext: DevServer) {
  const { app, config } = devSeverContext._context;
  if (!config.cors) return;
  app.use(cors(typeof config.cors === 'boolean' ? {} : config.cors));
}
