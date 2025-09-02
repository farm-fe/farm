function localWatch() {
  console.log("local watch");
}

function localIsRef() {
  console.log("local isRef");
}

export * from "./dep.js";
export { localWatch, localIsRef };
