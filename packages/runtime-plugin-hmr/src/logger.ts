export class Logger {
  debug(message: string) {
    console.debug('[Farm HMR]', message);
  }
  log(message: string) {
    console.log('[Farm HMR]', message);
  }
  warn(message: string) {
    console.warn('[Farm HMR] Warning:', message);
  }
  error(message: string) {
    console.warn('[Farm HMR] Error:', message);
  }
}

export const logger = new Logger();
