import { execSync } from "child_process";
import { buildCli, buildCoreCjs, buildJsPlugins } from "./build.mjs";

// Build node packages
await buildCli();
await buildCoreCjs();
await buildJsPlugins();

// Set npm config to public access
execSync("npm config set access public", { stdio: "inherit" });

// Publish nightly packages
execSync(`npx changeset publish --tag nightly`, { stdio: "inherit" });
