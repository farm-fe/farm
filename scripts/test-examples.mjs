import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";

import {
  buildCli,
  buildCoreCjs,
  buildExamples,
  buildJsPlugins,
  buildRuntime,
} from "./build.mjs";

function parseStartFromArg(argv) {
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

    if ((arg === "--from" || arg === "--start-from") && argv[i + 1]) {
      return argv[i + 1];
    }

    if (arg.startsWith("--from=")) {
      return arg.slice("--from=".length);
    }

    if (arg.startsWith("--start-from=")) {
      return arg.slice("--start-from=".length);
    }
  }

  return process.env.FARM_EXAMPLE_START_FROM;
}

function parseExampleArg(argv) {
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

    if (arg === "--example" && argv[i + 1]) {
      return argv[i + 1];
    }

    if (arg.startsWith("--example=")) {
      return arg.slice("--example=".length);
    }
  }

  return process.env.FARM_EXAMPLE;
}

function parseSkipBuildJsPluginsArg(argv) {
  for (const arg of argv) {
    if (arg === "--skip-build-js-plugins" || arg === "--skip-build-js-plugin") {
      return true;
    }

    if (arg.startsWith("--skip-build-js-plugins=")) {
      const value = arg.slice("--skip-build-js-plugins=".length).toLowerCase();
      return value === "1" || value === "true";
    }
  }

  const envValue = process.env.FARM_SKIP_BUILD_JS_PLUGINS;
  if (!envValue) {
    return false;
  }

  const normalizedEnvValue = envValue.toLowerCase();
  return normalizedEnvValue === "1" || normalizedEnvValue === "true";
}

const argv = process.argv.slice(2);
const startFrom = parseStartFromArg(argv);
const example = parseExampleArg(argv);
const skipBuildJsPlugins = parseSkipBuildJsPluginsArg(argv);

if (startFrom) {
  const examples = readdirSync("./examples").filter((name) =>
    statSync(join("./examples", name)).isDirectory(),
  );

  if (!examples.includes(startFrom)) {
    throw new Error(`Example '${startFrom}' was not found under ./examples`);
  }
}

if (example) {
  const examples = readdirSync("./examples").filter((name) =>
    statSync(join("./examples", name)).isDirectory(),
  );

  if (!examples.includes(example)) {
    throw new Error(`Example '${example}' was not found under ./examples`);
  }
}

console.log("Building CLI...");
await buildCli();
console.log("Building runtime...");
await buildRuntime();
console.log("Building core CJS...");
await buildCoreCjs();
if (skipBuildJsPlugins) {
  console.log("Skipping JS plugins build.");
} else {
  console.log("Building JS plugins...");
  await buildJsPlugins();
}

if (startFrom) {
  console.log(`Building examples from: ${startFrom}`);
}

if (example) {
  console.log(`Building only example: ${example}`);
}

await buildExamples({ startFrom, example });
