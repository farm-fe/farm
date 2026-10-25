import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

const CHANGESET_DIR = "./.changeset";

let changesetFilesExist = readdirSync(CHANGESET_DIR).filter((file) => file.endsWith(".md") && file !== 'README.md').map(file => file.replace(".md", ""));

// check pre release
const prePath = path.join(CHANGESET_DIR, "pre.json");

if (existsSync(prePath)) {
  const preRelease = JSON.parse(readFileSync(path.join(CHANGESET_DIR, "pre.json"), "utf-8"));
  // Adapted from 48de68386: consumed beta changesets still need stable versions on exit.
  if (preRelease.mode === 'exit') {
    console.log("Exiting prerelease mode, generate stable versions before publish");
    process.exit(0);
  }
  changesetFilesExist = changesetFilesExist.filter(file => !preRelease.changesets.includes(file));
}

if (changesetFilesExist.length === 0) {
  console.log("No changeset files found, try publish");
  process.exit(1);
} else {
  console.log("Changeset files found, skip publish");
  process.exit(0);
}

