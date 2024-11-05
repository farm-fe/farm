import {
  buildCli,
  buildCoreCjs,
  buildExamples,
  buildJsPlugins,
} from "./build.mjs";

console.log("Building CLI...");
await buildCli();
console.log("Building core CJS...");
await buildCoreCjs();
console.log("Building JS plugins...");
await buildJsPlugins();

// await buildExamples();
