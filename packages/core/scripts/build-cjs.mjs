import path from "path";
import { readFile, writeFile } from "fs/promises";

import { build } from "../dist/index.js";

await build({
  configPath: path.join(process.cwd(), "farm.config.ts"),
});

// replace local binary
const cjsFilePath = path.join(process.cwd(), "dist", "cjs", "index.cjs");
const cjsFileContent = await readFile(cjsFilePath, "utf8");

const content = cjsFileContent
  .replaceAll(/\.\/(farm\..+\.node)/g, "../../binding/$1")
  .replaceAll(/'(farm\..+\.node)'/g, "'../../binding/$1'");

await writeFile(cjsFilePath, content);
