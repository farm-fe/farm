import { execSync } from "child_process";

execSync(`npx changeset version --snapshot nightly`, {
  stdio: "inherit",
});

execSync("pnpm install --no-frozen-lockfile", { stdio: "inherit" });
