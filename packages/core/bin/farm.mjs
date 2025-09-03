#!/usr/bin/env node
if (process.argv.includes("--debug")) {
  process.env.DEBUG = "farm:*";
}

import("../dist/cli.js").then((module) => {
  module.runCli();
});
