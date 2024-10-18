import { execSync } from "child_process";
import { buildCli, buildCoreCjs, buildJsPlugins } from "./build.mjs";

// Generate nightly version number
const dateString =
  new Date().getFullYear() +
  String(new Date().getMonth() + 1).padStart(2, "0") +
  String(new Date().getDate()).padStart(2, "0");
const nightlyVersion = `2.0.0-nightly.${dateString}`;

// Build node packages
await buildCli();
await buildCoreCjs();
await buildJsPlugins();

// Set npm config to public access
execSync("npm config set access public", { stdio: "inherit" });

// Update versions to nightly
execSync(`npx changeset version --snapshot ${nightlyVersion}`, { stdio: "inherit" });

// Publish nightly packages
execSync(`npx changeset publish --tag ${nightlyVersion}`, { stdio: "inherit" });

console.log(`Nightly version ${nightlyVersion} published successfully.`);
