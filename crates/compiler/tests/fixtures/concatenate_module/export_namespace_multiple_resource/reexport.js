import * as dep from "./dep.js";

function localWatch() {
  dep.watch();
}

function localIsRef() {
  dep.isRef();
}

export * from "./dep.js";
export { localWatch, localIsRef };
