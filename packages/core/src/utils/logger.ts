import { fileURLToPath } from 'node:url';
import {
  PersistentCacheBrand,
  blue,
  bold,
  brandColor,
  cyan,
  debugColor,
  green,
  magenta,
  red,
  yellow
} from './color.js';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Config } from '../../binding/index.js';

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
  e?: Error;
  timestamp?: boolean;
  error?: Error;
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

export function printServerUrls(urls: any, logger: Logger): void {
  const colorUrl = (url: string) =>
    cyan(url.replace(/:(\d+)\//, (_, port) => `:${bold(port)}/`));

  const logUrl = (url: string, type: string) =>
    logger.info(`${bold(magenta('>'))} ${bold(type)}${bold(colorUrl(url))}`);

  urls.local.map((url: string) => logUrl(url, 'Local:   '));
  urls.network.map((url: string) => logUrl(url, 'Network: '));
}

export function bootstrapLogger(options?: LoggerOptions): Logger {
  return new DefaultLogger(options);
}

export function bootstrap(times: number, config: Config) {
  const usePersistentCache = config.config.persistentCache;
  const persistentCacheFlag = usePersistentCache
    ? bold(PersistentCacheBrand)
    : '';
  const version = JSON.parse(
    readFileSync(
      join(fileURLToPath(import.meta.url), '../../../package.json'),
      'utf-8'
    )
  ).version;
  console.log('\n', bold(brandColor(`${'ϟ'}  Farm  v${version}`)));
  console.log(
    `${bold(green(` ✓`))}  ${bold('Ready in')} ${bold(
      green(`${times}ms`)
    )} ${persistentCacheFlag}`,
    '\n'
  );
}
