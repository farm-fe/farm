import log from 'loglevel';
import chalk from 'chalk';

log.setDefaultLevel(log.levels.INFO);

export interface Logger {
  info(message: string, banner?: boolean): void;
  warn(message: string, banner?: boolean): void;
  error(message: string, banner?: boolean): void;
}
export function createLogger(): Logger {
  return {
    info: (message: string, banner = true) =>
      log.info(`${banner ? brandColor('[ Farm ] ') : ''}${message}`),
    warn: (message: string, banner = true) =>
      log.info(
        `${banner ? brandColor('[ Farm ] ') : ''}${chalk.yellowBright(message)}`
      ),
    error: (message: string, banner = true) =>
      log.info(
        `${banner ? brandColor('[ Farm ] ') : ''}${chalk.redBright(message)}`
      )
  };
}

export const brandColor = chalk.rgb(113, 26, 95);
