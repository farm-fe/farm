import { formatName } from './utils';
import { DEFAULT_PREFIX } from './constants';

export function createMessage(name: string): string {
  return DEFAULT_PREFIX + formatName(name);
}

export { formatName } from './utils';
export { DEFAULT_PREFIX } from './constants';
