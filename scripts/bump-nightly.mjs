import { execSync } from "child_process";

// generate nightly version
const dateString =
  new Date().getFullYear() +
  String(new Date().getMonth() + 1).padStart(2, "0") +
  String(new Date().getDate()).padStart(2, "0");
const nightlyVersion = `2.0.0-nightly.${dateString}`;

execSync(`npx changeset version --snapshot ${nightlyVersion}`, { stdio: "inherit" });

execSync("pnpm install --no-frozen-lockfile", { stdio: "inherit" });

console.log(`Nightly version bump completed. Version: ${nightlyVersion}`);
