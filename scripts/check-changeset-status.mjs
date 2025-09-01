import { readdirSync } from "node:fs";

const CHANGESET_DIR = "./.changeset";

const changesetFilesExist = readdirSync(CHANGESET_DIR).filter((file) => file.endsWith(".md") && file !== 'README.md');

if (changesetFilesExist.length === 0) {
  console.log("No changeset files found, try publish");
  process.exit(1);
} else {
  console.log("Changeset files found, skip publish");
  process.exit(0);
}

