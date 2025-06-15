import { Common1 } from "./common1";
import { Common3, Common2 }  from './route2-comp';
export function Route2() {
  return "Route2" + Common1() + Common2() + Common3();
}