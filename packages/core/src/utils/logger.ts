import log from 'loglevel';
import chalk from 'chalk';

log.setDefaultLevel(log.levels.INFO);

export const brandColor = chalk.rgb(113, 26, 95);

export interface Logger {
  trace(message: string): void;
  debug(message: string): void;
  info(message: string, banner?: boolean): void;
  warn(message: string): void;
  error(message: string | Error): void;
}

export class DefaultLogger implements Logger {
  constructor(private name: string = 'Farm') {}
  trace(message: string): void {
    log.trace(message);
  }
  debug(message: string): void {
    log.debug(message);
  }
  info(message: string, banner = true): void {
    log.info(
      `${banner ? brandColor(`[ ${this.name} ] `) : ''}${message} : ${message}`
    );
  }

  warn(message: string): void {
    log.warn(message);
  }

  error(message: string | Error): void {
    log.error(chalk.red(message));
  }
}
