export * from './dep.cjs';

import { Route1Comp } from "./route1-comp";
import { Common1 } from "./common1";

import { registerEngine } from "./dep.cjs";

export function Route1() {
  return Route1Comp() + Common1() + registerEngine('route1');
}