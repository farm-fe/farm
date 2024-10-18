import { execSync } from "child_process";
import { buildCli, buildCoreCjs, buildJsPlugins } from "./build.mjs";

// Generate nightly version number
const nightlyVersion = `0.0.0-nightly.${Date.now()}`;

// Build node packages
await buildCli();
await buildCoreCjs();
await buildJsPlugins();

// Set npm config to public access
execSync("npm config set access public", { stdio: "inherit" });

// Update versions to nightly
execSync(`npx changeset version --snapshot nightly`, { stdio: "inherit" });

// Publish nightly packages
execSync("npx changeset publish --tag nightly", { stdio: "inherit" });

console.log(`Nightly version ${nightlyVersion} published successfully.`);
