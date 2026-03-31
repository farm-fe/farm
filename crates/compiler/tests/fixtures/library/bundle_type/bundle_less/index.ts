import { formatName } from './lib/utils';
import { DEFAULT_PREFIX } from './lib/constants';

export function createMessage(name: string): string {
  return DEFAULT_PREFIX + formatName(name);
}

export { formatName } from './lib/utils';
export { DEFAULT_PREFIX } from './lib/constants';
