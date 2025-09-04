import { Server as DevServer } from '../server/index.js';
import type { PreviewServer } from '../server/preview.js';

export function safeJsonParse<T>(v: string, defaultValue?: T): T {
  try {
    return JSON.parse(v);
  } catch (error) {
    return defaultValue;
  }
}

export const isDevServer = (app: DevServer | PreviewServer): app is DevServer =>
  app instanceof DevServer;
