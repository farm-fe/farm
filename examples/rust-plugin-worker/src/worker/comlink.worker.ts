import * as Comlink from "comlink";

const api = {
  async add(a: number, b: number) {
    return a + b;
  },
};

Comlink.expose(api);
