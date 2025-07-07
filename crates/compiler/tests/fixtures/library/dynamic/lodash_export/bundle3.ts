import { Logger } from './logger';

export function bundle3() {
  const logger = new Logger();
  logger.log('bundle3');
}

export default 'default bundle3'