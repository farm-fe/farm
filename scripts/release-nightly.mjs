import { execSync } from "child_process";
import { buildCli, buildCoreCjs, buildJsPlugins } from "./build.mjs";

// Build node packages
await buildCli();
await buildCoreCjs();
await buildJsPlugins();

try {
  // Set npm config to public access
  execSync("npm config set access public", { stdio: "inherit" });

  // Publish nightly packages
  execSync(`npx changeset publish --no-git-tag --tag nightly`, {
    stdio: "inherit",
  });
} catch (error) {
  console.error(error);
  process.exit(1);
}
