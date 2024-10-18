import { execSync } from "child_process";

// generate nightly version
const nightlyVersion = `0.0.0-nightly.${Date.now()}`;

execSync(`npx changeset version --snapshot nightly`, { stdio: "inherit" });

execSync("pnpm install --no-frozen-lockfile", { stdio: "inherit" });

console.log(`Nightly version bump completed. Version: ${nightlyVersion}`);
