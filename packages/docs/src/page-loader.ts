import * as application from 'docs:application';
import { createPageLoader as create } from './page-loader-core.js';

export type { PageLoader } from './page-loader-core.js';
export function createPageLoader(initialKey?: string) {
  return create(application, initialKey);
}
