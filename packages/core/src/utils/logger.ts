import chalk, { ChalkInstance } from 'chalk';

export const brandColor = chalk.rgb(113, 26, 95);

type LogLevelNames = 'trace' | 'debug' | 'info' | 'warn' | 'error';

enum LogLevel {
  Trace = 'trace',
  Debug = 'debug',
  Info = 'info',
  Warn = 'warn',
  Error = 'error'
}

export interface Logger {
  trace(message: string): void;
  debug(message: string): void;
  info(message: string, showBanner?: boolean): void;
  warn(message: string): void;
  error(message: string | Error, options?: ErrorOptions): void;
}

export interface ErrorOptions {
  exit?: boolean;
}
interface LoggerOptions {
  name?: string;
  brandColor?: ChalkInstanceKeys;
}
type ChalkInstanceKeys = keyof ChalkInstance;
export class DefaultLogger implements Logger {
  constructor(
    public options?: LoggerOptions,
    private levelValues: Record<LogLevelNames, number> = {
      trace: 0,
      debug: 1,
      info: 2,
      warn: 3,
      error: 4
    },
    private prefix?: string
  ) {
    if (!options) {
      options = {};
    }
    const { name = 'Farm' } = options;

    const prefixColor: ChalkInstanceKeys = options.brandColor;
    this.prefix = options.brandColor
      ? chalk.hex(prefixColor)(`[ ${name} ] `)
      : brandColor(`[ ${name} ] `);
  }

  private logMessage(
    level: LogLevelNames,
    message: string,
    color?: ChalkInstance,
    showBanner = true
  ): void {
    if (this.levelValues[level] <= this.levelValues[level]) {
      const loggerMessage = color
        ? color(`${showBanner ? this.prefix : ''}${message}`)
        : `${showBanner ? this.prefix : ''}${message}`;
      console.log(loggerMessage);
    }
  }

  trace(message: string): void {
    this.logMessage(LogLevel.Trace, message, chalk.magenta);
  }

  debug(message: string): void {
    this.logMessage(LogLevel.Debug, message, chalk.blue);
  }

  info(message: string, showBanner?: boolean): void {
    this.logMessage(LogLevel.Info, message, null, showBanner);
  }

  warn(message: string): void {
    this.logMessage(LogLevel.Warn, message, chalk.yellow);
  }

  error(message: string | Error, options?: ErrorOptions): void {
    const errorMessage =
      message instanceof Error ? message.stack : `${message}`;
    this.logMessage(LogLevel.Error, errorMessage, chalk.red);

    if (options?.exit) {
      process.exit(1);
    }
  }
}
