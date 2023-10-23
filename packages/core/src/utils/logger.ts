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
    if (!this.options) this.options = {};
    this.brandPrefix();
  }

  private brandPrefix(color?: string | ChalkInstance) {
    const { name = 'Farm' } = this.options;

    let prefixColor: string | ChalkInstance | undefined;
    if (typeof this.options.brandColor === 'string') {
      prefixColor = this.options.brandColor;
    } else if (typeof color !== 'undefined') {
      prefixColor = color;
    }

    this.prefix = prefixColor
      ? typeof prefixColor === 'string'
        ? chalk.hex(prefixColor)(`[ ${name} ] `)
        : prefixColor(`[ ${name} ] `)
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
    this.brandPrefix(chalk.green);
    this.logMessage(LogLevel.Trace, message, chalk.magenta);
  }

  debug(message: string): void {
    this.brandPrefix('#ff8c00');
    this.logMessage(LogLevel.Debug, message, chalk.blue);
  }

  info(message: string, showBanner?: boolean): void {
    this.brandPrefix();
    this.logMessage(LogLevel.Info, message, null, showBanner);
  }

  warn(message: string): void {
    this.brandPrefix(chalk.yellowBright);
    this.logMessage(LogLevel.Warn, message, chalk.yellow);
  }

  error(message: string | Error, options?: ErrorOptions): void {
    this.brandPrefix(chalk.red);
    const errorMessage =
      message instanceof Error ? message.stack : `${message}`;
    this.logMessage(LogLevel.Error, errorMessage, chalk.red);

    if (options?.exit) {
      process.exit(1);
    }
  }
}

export function printServerUrls(
  urls: any,
  optionsHost: string | boolean | undefined,
  info: Logger['info']
): void {
  const colorUrl = (url: string) =>
    chalk.cyan(url.replace(/:(\d+)\//, (_, port) => `:${chalk.bold(port)}/`));
  for (const url of urls.local) {
    info(`  ${chalk.green('➜')}  ${chalk.bold('Local')}:   ${colorUrl(url)}`);
  }
  for (const url of urls.network) {
    info(`  ${chalk.green('➜')}  ${chalk.bold('Network')}: ${colorUrl(url)}`);
  }
  if (urls.network.length === 0 && optionsHost === undefined) {
    info(
      chalk.dim(`  ${chalk.green('➜')}  ${chalk.bold('Network')}: use `) +
        chalk.bold('--host') +
        chalk.dim(' to expose')
    );
  }
}
