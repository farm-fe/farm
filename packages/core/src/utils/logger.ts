import { Config } from '../types/binding.js';
import { ColorFunction, PersistentCacheBrand, colors } from './color.js';
/* eslint-disable @typescript-eslint/no-explicit-any */
import { pad, version } from './share.js';

type LogLevelNames = 'trace' | 'debug' | 'info' | 'warn' | 'error';

enum LogLevel {
  Trace = 'trace',
  Debug = 'debug',
  Info = 'info',
  Warn = 'warn',
  Error = 'error'
}

export interface ILogger {
  trace(message: string): void;
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  warnOnce(message: string): void;
  errorOnce(message: string | Error): void;
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

export class Logger implements ILogger {
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
      const loggerMessage = color ? color(prefix + message) : prefix + message;
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

    const effectiveOptions = { ...this.options, ...errorOptions };
    const causeError = errorOptions?.e || errorOptions?.error;

    let error;

    if (typeof message === 'string') {
      error = new Error(message);
      error.stack = '';
    } else {
      error = message;
    }

    if (causeError) {
      error.message += `\nCaused by: ${causeError.stack ?? causeError}`;
    }

    this.logMessage(LogLevel.Error, error, colors.red);

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
  hasErrorLogged(message: string | Error) {
    return errorOnceMessages.has(message);
  }
  hasWarnLogged(message: string) {
    return warnOnceMessages.has(message);
  }
}

// use in test
// TODO: impl ILogger
export class NoopLogger extends Logger {
  setPrefix(_options: LoggerOptions): void {}
  trace(_message: string): void {}
  debug(_message: string): void {}
  info(_message: string, _iOptions?: LoggerOptions): void {}
  warn(_message: string): void {}
  error(_message: string | Error, _errorOptions?: ErrorOptions): void {
    if (_errorOptions.exit) {
      let e = _message instanceof Error ? _message : new Error(_message);
      if (_errorOptions?.e || _errorOptions?.error) {
        e.cause = _errorOptions.e || _errorOptions.error;
      }

      throw e;
    }
  }
  infoOnce(_message: string): void {}
  warnOnce(_message: string): void {}
  errorOnce(_message: string | Error): void {}
  hasErrorLogged(_message: string | Error): boolean {
    return false;
  }
  hasWarnLogged(_message: string): boolean {
    return false;
  }
}

export function printServerUrls(
  urls: any,
  logger: Logger,
  previewFlag = false
): void {
  if (previewFlag)
    logger.info(colors.bold(colors.magenta('preview server running at: \n')));
  const colorUrl = (url: string) =>
    colors.cyan(url.replace(/:(\d+)\//, (_, port) => `:${colors.bold(port)}/`));

  const logUrl = (url: string, type: string) =>
    logger.info(
      `${colors.bold(colors.magenta('>'))} ${colors.bold(type)}${colors.bold(
        colorUrl(url)
      )}`
    );

  urls.local.map((url: string) => logUrl(url, 'Local:   '));
  urls.network.map((url: string) => logUrl(url, 'Network: '));
}

export function bootstrapLogger(options?: LoggerOptions): Logger {
  return new Logger(options);
}

export function bootstrap(times: number, config: Config) {
  const usePersistentCache = config.config.persistentCache;
  const persistentCacheFlag = usePersistentCache
    ? colors.bold(PersistentCacheBrand)
    : '';

  console.log(
    '\n',
    colors.bold(colors.brandColor(`${'ϟ'}  Farm  v${version}`))
  );
  console.log(
    `${colors.bold(colors.green(` ✓`))}  ${colors.bold(
      'Ready in'
    )} ${colors.bold(colors.green(`${times}ms`))} ${persistentCacheFlag}`,
    '\n'
  );
}

export const logger = new Logger();

export function buildErrorMessage(
  err: any,
  args: string[] = [],
  includeStack = true
): string {
  if (err.plugin) args.push(`  Plugin: ${colors.magenta(err.plugin)}`);
  const loc = err.loc ? `:${err.loc.line}:${err.loc.column}` : '';
  if (err.id) args.push(`  File: ${colors.cyan(err.id)}${loc}`);
  if (err.frame) args.push(colors.yellow(pad(err.frame)));
  if (includeStack && err.stack) args.push(pad(cleanStack(err.stack)));
  return args.join('\n');
}

function cleanStack(stack: string) {
  return stack
    .split(/\n/g)
    .filter((l) => /^\s*at/.test(l))
    .join('\n');
}
