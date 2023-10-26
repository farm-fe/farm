import {
  blue,
  bold,
  brandColor,
  cyan,
  debugColor,
  dim,
  green,
  magenta,
  red,
  yellow
} from './color.js';

type LogLevelNames = 'trace' | 'debug' | 'info' | 'warn' | 'error';

enum LogLevel {
  Trace = 'trace',
  Debug = 'debug',
  Info = 'info',
  Warn = 'warn',
  Error = 'error'
}

export interface Logger {
  trace(message: string | string[]): void;
  debug(message: string | string[]): void;
  info(message: string | string[]): void;
  warn(message: string | string[]): void;
  error(message: string | string[] | Error[], options?: ErrorOptions): void;
}

export interface ErrorOptions {
  exit?: boolean;
}
interface LoggerOptions {
  name?: string;
  brandColor?: any;
  exit?: boolean;
}

const LOGGING_METHOD = {
  info: 'log',
  warn: 'warn',
  error: 'error'
} as const;

const warnOnceMessages = new Set();
const infoOnceMessages = new Set();
const errorOnceMessages = new Set();

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

  private brandPrefix(color?: (s: string | string[]) => string): void {
    const { name = 'Farm' } = this.options;
    const formattedName = bold(name);
    const formattedPrefix = bold(`[ ${formattedName} ]`);
    this.prefix = color ? color(formattedPrefix) : formattedPrefix;
  }

  private logMessage(
    level: LogLevelNames,
    message: any[],
    color?: any,
    showBanner = true
  ): void {
    const loggerMethod =
      level in LOGGING_METHOD
        ? LOGGING_METHOD[level as keyof typeof LOGGING_METHOD]
        : 'log';
    if (this.levelValues[level] <= this.levelValues[level]) {
      const loggerMessage = color
        ? color(`${showBanner ? this.prefix : ''} ${message}`)
        : `${showBanner ? this.prefix : ''} ${message}`;
      console[loggerMethod](loggerMessage);
    }
  }

  trace(...message: any[]): void {
    this.brandPrefix(green);
    this.logMessage(LogLevel.Trace, message, magenta);
  }

  debug(...message: any[]): void {
    this.brandPrefix(debugColor);
    this.logMessage(LogLevel.Debug, message, blue);
  }

  info(...message: any[]): void {
    this.brandPrefix(brandColor);
    this.logMessage(LogLevel.Info, message, null);
  }

  warn(...message: any[]): void {
    this.brandPrefix(yellow);
    this.logMessage(LogLevel.Warn, message, yellow);
  }

  error(...message: any[]): void {
    this.brandPrefix(red);
    this.logMessage(LogLevel.Error, message, red);

    if (this.options?.exit) {
      process.exit(1);
    }
  }
  infoOnce(...message: any[]) {
    if (!warnOnceMessages.has(message[0])) {
      infoOnceMessages.add(message.join(' '));
      this.info(...message);
    }
  }
  warnOnce(...message: any[]) {
    if (!warnOnceMessages.has(message[0])) {
      warnOnceMessages.add(message.join(' '));
      this.warn(...message);
    }
  }
  errorOnce(...message: any[]) {
    if (!warnOnceMessages.has(message[0])) {
      errorOnceMessages.add(message.join(' '));
      this.error(...message);
    }
  }
}

export function printServerUrls(
  urls: any,
  optionsHost: string | boolean | undefined,
  logger: Logger
): void {
  const colorUrl = (url: string) =>
    cyan(url.replace(/:(\d+)\//, (_, port) => `:${bold(port)}/`));
  for (const url of urls.local) {
    logger.info(`${magenta('➡️')} ${bold('Local')}:   ${bold(colorUrl(url))}`);
  }
  for (const url of urls.network) {
    logger.info(`${magenta('➡️')} ${bold('Network')}: ${bold(colorUrl(url))}`);
  }
  if (urls.network.length === 0 && optionsHost === undefined) {
    logger.info(
      dim(`${magenta('➡️')} ${bold('Network')}: use `) +
        bold('--host') +
        dim(' to expose')
    );
  }
}
