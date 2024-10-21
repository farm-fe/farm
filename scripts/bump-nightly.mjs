import { execSync } from "child_process";

// generate nightly version
const dateString =
  new Date().getFullYear() +
  String(new Date().getMonth() + 1).padStart(2, "0") +
  String(new Date().getDate()).padStart(2, "0");

const gitHash = execSync("git rev-parse --short HEAD").toString().trim();

const nightlyVersion = `2.0.0-nightly.${dateString}.${gitHash}`;

execSync(`npx changeset version --snapshot ${nightlyVersion}`, {
  stdio: "inherit",
});

execSync("pnpm install --no-frozen-lockfile", { stdio: "inherit" });
