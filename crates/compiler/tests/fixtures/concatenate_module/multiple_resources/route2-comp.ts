export * from './common3';
export { Common2 } from './common1';

import * as R from "./route2";

export function Route2Comp() {
  return R.dep2('Route2Comp')
}
