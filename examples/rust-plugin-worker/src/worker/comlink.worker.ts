import * as Comlink from "comlink";
import { count } from "./util";
export class MyWorker {
  async add(a: number, b: number) {
    return count(a, b);
  }
}

Comlink.expose(MyWorker);
