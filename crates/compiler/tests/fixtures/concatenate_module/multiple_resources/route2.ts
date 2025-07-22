export * from './dep2.cjs';

import { Common1 } from "./common1";
import { Common3, Common2, Route2Comp }  from './route2-comp';

export function Route2() {
  return "Route2" + Common1() + Common2() + Common3() + Route2Comp();
}