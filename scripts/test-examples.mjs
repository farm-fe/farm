import {
  buildCorePkg,
  buildCoreCjs,
  buildExamples,
  buildJsPlugins,
  buildRuntime,
} from "./build.mjs";

console.log("Building core package...");
await buildCorePkg();
console.log("Building runtime...");
await buildRuntime();
console.log("Building core CJS...");
await buildCoreCjs();
console.log("Building JS plugins...");
await buildJsPlugins();

await buildExamples();
