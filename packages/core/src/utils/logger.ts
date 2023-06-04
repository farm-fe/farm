import log from 'loglevel';
import chalk from 'chalk';

export const brandColor = chalk.rgb(113, 26, 95);

type LogLevelNames = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface Logger {
  trace(message: string): void;
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  error(message: string | Error): void;
}

export class DefaultLogger implements Logger {
  private prefix: string;

  constructor(
    private name = 'Farm',
    level: LogLevelNames = 'info',
    private levelValues: Record<LogLevelNames, number> = {
      trace: 0,
      debug: 1,
      info: 2,
      warn: 3,
      error: 4
    }
  ) {
    log.setDefaultLevel(level);
    console.log(`${log.getLevel()} ${level}`);
    this.prefix = brandColor(`[ ${this.name} ] `);
  }

  private logMessage(
    level: LogLevelNames,
    message: string,
    color?: any,
    showBanner?: boolean
  ): void {
    const userLevel = log.getLevel();
    if (userLevel <= this.levelValues[level]) {
      console.log(color(`${showBanner ? this.prefix : ''}${message}`));
    }
  }

  trace(message: string): void {
    this.logMessage('trace', message, chalk.gray);
  }

  debug(message: string): void {
    this.logMessage('debug', message, chalk.blue);
  }

  info(message: string): void {
    this.logMessage('info', message);
  }

  warn(message: string): void {
    this.logMessage('warn', message, chalk.yellow);
  }

  error(message: string | Error): void {
    this.logMessage('error', `${message}`, chalk.red);
  }
}
