export class Logger {
  log(message: string) {
    console.log('[Farm HMR]', message);
  }
  warn(message: string) {
    console.warn('[Farm HMR] Warning:', message);
  }
}

export const logger = new Logger();
