import { readFile, writeFile } from "fs/promises";
import path from "path";

const filePaths = [path.join(process.cwd(), "src", "main.tsx"), path.join(process.cwd(), "src", "index.tsx"), path.join(process.cwd(), "src", "comps", "title", "index.tsx")];

for (const filePath of filePaths) {
  const file = await readFile(filePath, "utf-8");

  if (filePath === path.join(process.cwd(), "src", "index.tsx")) {
    await writeFile(filePath, file.concat('module.meta.hot.accept(() => window.location.reload())'));
  } else {
    await writeFile(filePath, file.concat(`console.log('${filePath} updated')`));
  }
}