export class Logger {
  log(message: string) {
    console.log('[Farm HMR]', message);
  }
}

export const logger = new Logger();
