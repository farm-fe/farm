import { execSync } from "child_process";

// build node packages
execSync("npm run build", { stdio: "inherit" });
// publish node packages
execSync("npx changeset publish", { stdio: "inherit" });

// publish all packages under packages/core/npm
// TODO: investigate why napi prepublish doesn't work
const packages = [
  'darwin-x64',
  'darwin-arm64',
  'linux-x64-gnu',
  'win32-x64-msvc',
];

packages.forEach((pkg) => {
  execSync(`npm publish packages/core/npm/${pkg}`, { stdio: "inherit" });
});