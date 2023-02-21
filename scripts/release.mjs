import { execSync } from "child_process";

// build node packages
execSync("npm run build", { stdio: "inherit" });
execSync("npm config set access public", { stdio: "inherit" });
// publish node packages
execSync("npx changeset publish", { stdio: "inherit" });