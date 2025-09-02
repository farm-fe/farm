import { execSync } from "child_process";
import { buildCorePkg, buildCoreCjs, buildJsPlugins, buildRuntime } from './build.mjs';

// Build node packages
await buildCorePkg();
await buildJsPlugins();
await buildCoreCjs();
await buildRuntime();

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
