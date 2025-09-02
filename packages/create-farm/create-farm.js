#!/usr/bin/env node

const cli = require("./index");
const path = require("path");

const [bin, script, ...args] = process.argv;
const binStem = path.parse(bin).name.toLowerCase();

let binName;
if (binStem.match(/(nodejs|node|bun)-*([0-9]*)*$/g)) {
  const managerStem = process.env.npm_execpath
    ? path.parse(process.env.npm_execpath).name.toLowerCase()
    : null;
  if (managerStem) {
    let manager;
    switch (managerStem) {
      // Only supported package manager that has a different filename is npm.
      case "npm-cli":
        manager = "npm";
        break;

      // Yarn, pnpm, and Bun have the same stem name as their bin.
      // We assume all unknown package managers do as well.
      default:
        manager = managerStem;
        break;
    }

    binName = `${manager} run ${process.env.npm_lifecycle_event}`;
  } else {
    // Assume running NodeJS if we didn't detect a manager from the env.
    // We normalize the path to prevent the script's absolute path being used.
    const scriptNormal = path.normalize(path.relative(process.cwd(), script));
    binName = `${binStem} ${scriptNormal}`;
  }
} else {
  args.unshift(bin);
}

function pkgManagerFromUserAgent(userAgent) {
  if (!userAgent) return undefined;
  return userAgent.split(" ")[0]?.split("/")[0];
}

const pkgManager = pkgManagerFromUserAgent(process.env.npm_config_user_agent);
cli.run(args, binName, pkgManager);
