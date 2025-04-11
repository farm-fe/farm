import { __FARM_GLOBAL__ } from '../config/_global.js';
import { ResolvedUserConfig } from '../config/types.js';
import {
  ColorFunction,
  PersistentCacheBrand,
  bold,
  colors,
  green
} from './color.js';
import { ResolvedServerUrls } from './http.js';
import { getShortName } from './path.js';
import { clearScreen, pad, version } from './share.js';

type LogLevelNames = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface ILogger {
  trace(message: string, clearScreen?: Boolean): void;
  debug(message: string, clearScreen?: Boolean): void;
  info(message: string, clearScreen?: Boolean): void;
  warn(message: string, clearScreen?: Boolean): void;
  warnOnce(message: string, clearScreen?: Boolean): void;
  errorOnce(message: string | Error, clearScreen?: Boolean): void;
  error(
    message: string | Error,
    options?: ErrorOptions,
    clearScreen?: Boolean
  ): void;
}

export interface ErrorOptions {
  exit?: boolean;
  e?: Error;
  error?: Error;
}
interface LoggerOptions {
  prefix?: string;
  customLogger?: Logger;
  allowClearScreen?: boolean;
  brandColor?: ColorFunction;
  exit?: boolean;
  timeUnit?: 's' | 'ms';
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
  prefix: string;
  canClearScreen: boolean;
  colorMap: {
    trace: (input: string) => string;
    debug: (input: string) => string;
    info: (input: string) => string;
    warn: (input: string) => string;
    error: (input: string) => string;
  };

  private clear: () => void = () => {};
  private customLogger?: Logger;
  private timeUnit: 's' | 'ms';

  constructor(
    {
      prefix = 'Farm',
      allowClearScreen = true,
      customLogger,
      timeUnit = 'ms',
      brandColor
    }: LoggerOptions = {},
    private levelValues: Record<LogLevelNames, number> = {
      trace: 0,
      debug: 1,
      info: 2,
      warn: 3,
      error: 4
    }
  ) {
    this.canClearScreen =
      allowClearScreen && process.stdout.isTTY && !process.env.CI;
    this.clear = this.canClearScreen ? clearScreen : () => {};
    this.colorMap = {
      trace: colors.green,
      debug: colors.debugColor,
      info: brandColor ?? colors.brandColor,
      warn: colors.yellow,
      error: colors.red
    };
    this.prefix = prefix;
    this.customLogger = customLogger;
    this.timeUnit = timeUnit;
    this.brandPrefix();
  }

  private brandPrefix(color?: (s: string | string[]) => string): void {
    const formattedName = colors.bold(this.prefix);
    const formattedPrefix = colors.bold(`[ ${formattedName} ]`);
    this.prefix = color ? color(formattedPrefix) : formattedPrefix;
  }

  formatTime(duration: number): string {
    if (this.timeUnit === 's') {
      return `${(duration / 1000).toFixed(3)}s`;
    } else {
      return `${Math.floor(duration)}ms`;
    }
  }

  private logMessage(
    level: LogLevelNames,
    message: string | Error,
    color?: (s: string | string[]) => string,
    clearScreen = false,
    showBanner = true
  ): void {
    if (this.customLogger) {
      this.customLogger.logMessage(level, message, color, clearScreen);
      return;
    }

    const minLevel = process.env.LOG_LEVEL || 'info';
    if (
      this.levelValues[level] >= this.levelValues[minLevel as LogLevelNames]
    ) {
      if (this.canClearScreen && clearScreen) {
        this.clear();
      }
      const prefix = showBanner ? `${this.prefix} ` : '';
      const prefixColored = this.colorMap[level](prefix);
      let loggerMessage: string;

      if (typeof message === 'string') {
        const timeRegex = new RegExp(`\\{time:(\\d+(\\.\\d+)?)\\}`, 'g');
        loggerMessage = message.replace(timeRegex, (_, durationStr) => {
          const duration = parseFloat(durationStr);

          return this.formatTime(duration);
        });
      } else {
        loggerMessage = message.message;
      }
      loggerMessage = color ? color(loggerMessage) : loggerMessage;

      console.log(prefixColored + loggerMessage);
    }
  }

  setPrefix(options: LoggerOptions): void {
    if (options.prefix) {
      this.prefix = options.prefix;
      this.brandPrefix(options.brandColor);
    }
  }

  trace(message: string, clearScreen = false): void {
    this.logMessage('trace', message, colors.magenta, clearScreen);
  }

  debug(message: string, clearScreen = false): void {
    this.logMessage('debug', message, colors.blue, clearScreen);
  }

  info(message: string, clearScreen = false): void {
    this.logMessage('info', message, null, clearScreen);
  }

  warn(message: string, clearScreen = false): void {
    this.logMessage('warn', message, colors.yellow, clearScreen);
  }

  error(
    message: string | Error,
    errorOptions?: ErrorOptions,
    clearScreen = false
  ): void {
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

    this.logMessage('error', error, colors.red, clearScreen);
  }

  infoOnce(message: string, clearScreen = false): void {
    if (!infoOnceMessages.has(message)) {
      infoOnceMessages.add(message);
      this.info(message, clearScreen);
    }
  }

  warnOnce(message: string, clearScreen = false): void {
    if (!warnOnceMessages.has(message)) {
      warnOnceMessages.add(message);
      this.warn(message, clearScreen);
    }
  }

  errorOnce(message: string | Error, clearScreen = false): void {
    if (!errorOnceMessages.has(message)) {
      errorOnceMessages.add(message);
      this.error(message, undefined, clearScreen);
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
  info(_message: string): void {}
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

export function bootstrapLogger(options?: LoggerOptions): Logger {
  return new Logger(options);
}

export function bootstrap(
  time: number,
  config: ResolvedUserConfig,
  hasCacheDir: boolean
): void {
  if (!__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__) {
    const shortFile = getShortName(config.configFilePath, config.root);
    config.logger.info(`Using config file at ${bold(green(shortFile))}`, true);
  }
  const hasPersistentCache = config.compilation.persistentCache && hasCacheDir;
  const persistentCacheFlag = hasPersistentCache
    ? colors.bold(PersistentCacheBrand)
    : '';

  console.log(
    '\n',
    colors.bold(colors.brandColor(`${'ϟ'}  Farm  v${version}`))
  );

  console.log(
    `${colors.bold(colors.green(` ✓`))}  ${colors.bold(
      'Compile in'
    )} ${colors.bold(
      colors.green(config.logger.formatTime(time))
    )} ${persistentCacheFlag}`,
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

export function printServerUrls(
  urls: ResolvedServerUrls,
  optionsHost: string | boolean | undefined,
  logger: ILogger
): void {
  const colorUrl = (url: string) =>
    colors.cyan(url.replace(/:(\d+)\//, (_, port) => `:${colors.bold(port)}/`));
  for (const url of urls.local) {
    logger.info(
      `${colors.bold(colors.green('➜ '))} ${colors.bold(
        'Local'
      )}:   ${colors.bold(colorUrl(url))}`
    );
  }
  for (const url of urls.network) {
    logger.info(
      `${colors.bold(colors.green('➜ '))} ${colors.bold(
        'Network'
      )}: ${colors.bold(colorUrl(url))}`
    );
  }
  if (urls.network.length === 0 && optionsHost === undefined) {
    logger.info(
      colors.dim(`  ${colors.green('➜ ')}  ${colors.bold('Network')}: use `) +
        colors.bold('--host') +
        colors.dim(' to expose')
    );
  }
}
