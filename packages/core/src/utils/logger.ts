/* eslint-disable @typescript-eslint/no-explicit-any */
import { fileURLToPath } from 'node:url';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import {
  ColorFunction,
  PersistentCacheBrand,
  colors
} from './color.js';
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
  trace(message: string): void;
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  error(message: string | Error, options?: ErrorOptions): void;
}

export interface ErrorOptions {
  exit?: boolean;
  e?: Error;
  error?: Error;
}
interface LoggerOptions {
  name?: string;
  brandColor?: ColorFunction;
  exit?: boolean;
}

const LOGGER_METHOD = {
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
    const formattedName = colors.bold(name);
    const formattedPrefix = colors.bold(`[ ${formattedName} ]`);
    this.prefix = color ? color(formattedPrefix) : formattedPrefix;
  }

  private logMessage(
    level: LogLevelNames,
    message: string | Error,
    color?: (s: string | string[]) => string,
    showBanner = true
  ): void {
    const loggerMethod =
      level in LOGGER_METHOD
        ? LOGGER_METHOD[level as keyof typeof LOGGER_METHOD]
        : 'log';
    if (this.levelValues[level] <= this.levelValues[level]) {
      const prefix = showBanner ? this.prefix + ' ' : '';
      const loggerMessage = color
        ? color(prefix + message)
        : prefix + message;
      console[loggerMethod](loggerMessage);
    }
  }

  setPrefix(options: LoggerOptions): void {
    if (options.name) {
      this.options.name = options.name;
      this.brandPrefix(options.brandColor);
    }
  }


  trace(message: string): void {
    this.brandPrefix(colors.green);
    this.logMessage(LogLevel.Trace, message, colors.magenta);
  }

  debug(message: string): void {
    this.brandPrefix(colors.debugColor);
    this.logMessage(LogLevel.Debug, message, colors.blue);
  }


  info(message: string, iOptions?: LoggerOptions): void {
    const options: LoggerOptions | undefined = iOptions;
    if (options) {
      this.setPrefix(options);
    }
    if (!options || !options.brandColor) {
      this.brandPrefix(colors.brandColor);
    }
    this.logMessage(LogLevel.Info, message, null);
  }

  warn(message: string): void {
    this.brandPrefix(colors.yellow);
    this.logMessage(LogLevel.Warn, message, colors.yellow);
  }

  error(message: string | Error, errorOptions?: ErrorOptions): void {
    this.brandPrefix(colors.red);
    this.logMessage(LogLevel.Error, message, colors.red);

    const effectiveOptions = { ...this.options, ...errorOptions };
    
    if (effectiveOptions.exit) {
      process.exit(1);
    }
  }
  infoOnce(message: string) {
    if (!infoOnceMessages.has(message)) {
      infoOnceMessages.add(message);
      this.info(message);
    }
  }
  warnOnce(message: string) {
    if (!warnOnceMessages.has(message)) {
      warnOnceMessages.add(message);
      this.warn(message);
    }
  }
  errorOnce(message: string | Error) {
    if (!errorOnceMessages.has(message)) {
      errorOnceMessages.add(message);
      this.error(message);
    }
  }
}

export function printServerUrls(urls: any, logger: Logger): void {
  const colorUrl = (url: string) =>
  colors.cyan(url.replace(/:(\d+)\//, (_, port) => `:${colors.bold(port)}/`));

  const logUrl = (url: string, type: string) =>
    logger.info(`${colors.bold(colors.magenta('>'))} ${colors.bold(type)}${colors.bold(colorUrl(url))}`);

  urls.local.map((url: string) => logUrl(url, 'Local:   '));
  urls.network.map((url: string) => logUrl(url, 'Network: '));
}

export function bootstrapLogger(options?: LoggerOptions): Logger {
  return new DefaultLogger(options);
}

export function bootstrap(times: number, config: Config) {
  const usePersistentCache = config.config.persistentCache;
  const persistentCacheFlag = usePersistentCache
    ? colors.bold(PersistentCacheBrand)
    : '';
  const version = JSON.parse(
    readFileSync(
      join(fileURLToPath(import.meta.url), '../../../package.json'),
      'utf-8'
    )
  ).version;
  console.log('\n', colors.bold(colors.brandColor(`${'ϟ'}  Farm  v${version}`)));
  console.log(
    `${colors.bold(colors.green(` ✓`))}  ${colors.bold('Ready in')} ${colors.bold(
      colors.green(`${times}ms`)
    )} ${persistentCacheFlag}`,
    '\n'
  );
}
